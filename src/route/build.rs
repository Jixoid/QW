use std::time::Instant;

use owo_colors::OwoColorize;
use crate::{BuildVariant, ast, error::{CompilerError, Result}, front::Front, hgen::HGen};


pub struct BuildInfo<'a> {
  pub path: &'a str,
  pub variant: BuildVariant,
  pub verbose: bool,
  pub timings: bool,
  pub usages: bool,
  pub check_only: bool,
}


pub struct FileArena {
  files: std::cell::RefCell<Vec<Box<ast::Module>>>,
  loaded_paths: std::cell::RefCell<std::collections::HashSet<String>>,
}

impl FileArena {
  pub fn new() -> Self { Self { files: std::cell::RefCell::new(Vec::new()), loaded_paths: std::cell::RefCell::new(std::collections::HashSet::new()) } }
  
  pub fn alloc(&self, mut file: ast::Module) -> &ast::Module {
    let fid = self.files.borrow().len();
    assert!(fid < 0xFFFF_FFFF);

    file.fid = fid as u16;

    let b = Box::new(file);
    let ptr = &*b as *const ast::Module;
    self.loaded_paths.borrow_mut().insert(b.fpath.clone());
    
    self.files.borrow_mut().push(b);
    unsafe { &*ptr }
  }

  pub fn get<'a>(&'a self, fid: u16) -> &'a ast::Module {
    let ptr = &*self.files.borrow()[fid as usize] as &ast::Module as *const ast::Module;
    unsafe { &*ptr }
  }

  pub fn is_loaded(&self, fpath: &str) -> bool {
    self.loaded_paths.borrow().contains(fpath)
  }
}


fn humanize_size(size: usize) -> String {
  let set = 'a: {
    let mut i = 0;
    let mut size = size as f64;

    loop {
      if size > 1024.0 { size /= 1024.0; i += 1; }
      else { break 'a (size, i); }
    }
  };

  let letter = match set.1 {
    0 => "b",
    1 => "kb",
    2 => "mb",
    3 => "gb",
    4 => "tb",
    _ => todo!()
  };

  format!("{:.2}{}", set.0, letter)
}



pub fn build_cre<'a>(info: &BuildInfo, farena: &'a FileArena, fpath: String) -> Result<()> {
  let mut ast_cre = ast::Crate::new();

  let mol = farena.alloc(ast::Module::new(&fpath)?);
  
  let mut anys = vec![];
  
  let mut now: Instant;
  
  // Front
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "front".red().bold(), ":".bright_black(), mol.name) }
    
    let sum = Front::new(&mut ast_cre, &mol, farena)?.parse(&mut anys);

    let root = ast::Item{vis: ast::Visibility::Public, vari: ast::ItemVari::Module{ name: "".to_string(), ctn: ast_cre.new_extra(anys) }};
    ast_cre.root = ast_cre.new_item(root);

    /* time */ let front_time = now.elapsed();
    /* usag */ let ast_usage = ast_cre.storage_size();
    
    // Summary
    for m in sum.msgs() { eprintln!("{}", m.display(farena)); }
    if sum.sumall() > 0 { eprintln!("{}", sum); }
    if sum.sumerr() > 0 { return Ok(()); }
    drop(sum);


  // HGen
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "hgen".red().bold(), ":".bright_black(), mol.name) }
    
    let hir_cre = HGen::lower(&ast_cre);
    
    /* time */ let hgen_time = now.elapsed();
    /* usag */ let hir_usage = hir_cre.storage_size();

  /*
  // Sema
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "sema".red().bold(), ":".bright_black(), mol.name) }
    
    let sum = Sema::new(&mut mol).check();
    
    /* time */ let sema = now.elapsed();

    // Summary
    for m in sum.msgs() { eprintln!("{}", m); }
    if sum.sumall() > 0 { eprintln!("{}", sum); }
    if sum.sumerr() > 0 { return Ok(()); }
    drop(sum);
  
    if info.check_only {
      if info.timings {
        println!("{}{} {:?}", "total-time".yellow().bold(), ":".bright_black(), (front+sema));
        if info.verbose {
          println!("  {}{} {:?}", "front".blue().bold(), ":".bright_black(), front);
          println!("  {}{}  {:?}", "sema".blue().bold(), ":".bright_black(), sema);
        }
      }
      return Ok(());
    }


  // CGen
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "cgen".red().bold(), ":".bright_black(), mol.name) }
    
    let bin = crate::cgen::CGen::new(&hir, crate::basic_cgen::BasicCGen::new(&target)).generate();
  
    /* time */ let cgen = now.elapsed();

  let out_dir = std::path::Path::new(info.path).join("build");
  let _ = std::fs::create_dir_all(&out_dir);
  let out_file = out_dir.join("out.ll");
  let _ = std::fs::write(&out_file, &bin);
  */

  // Time
  if info.timings {
    println!("{}{} {:?}", "total-time".yellow().bold(), ":".bright_black(), (front_time+hgen_time/*+sema+cgen*/));
    
    if info.verbose {
      println!("  {}{} {:?}", "front".blue().bold(), ":".bright_black(), front_time);
      println!("  {}{}  {:?}", "hgen".blue().bold(), ":".bright_black(), hgen_time);
      /*
      println!("  {}{}  {:?}", "sema".blue().bold(), ":".bright_black(), sema);
      println!("  {}{}  {:?}", "cgen".blue().bold(), ":".bright_black(), cgen);
      */
    }
  }

  // Usage
  if info.usages {
    println!("{}{} {}", "mem-usage".yellow().bold(), ":".bright_black(), humanize_size(ast_usage+hir_usage));

    if info.verbose {
      println!("  {}{} {}", "ast".blue().bold(), ":".bright_black(), humanize_size(ast_usage));
      println!("  {}{} {}", "hir".blue().bold(), ":".bright_black(), humanize_size(hir_usage));
    }
  }

  Ok(())
}


pub fn build(info: BuildInfo) -> Result<()> {
  let conf_path = std::path::Path::new(info.path).join("qw.conf");
  if !conf_path.exists() {
    return Err(CompilerError::Str("could not find `qw.conf`.".to_string()));
  }

  //let mfd = Module::new(conf_path.to_str().unwrap_or(""))?;

  //let conf = crate::ds::Value::load_file(&mfd).map_err(|e| CompilerError::Str(format!("{:?}", e)))?;

  //let crate_name = || -> String {
  //  if let crate::ds::Value::Stc(stc) = conf {
  //    for field in &stc.subs {
  //      if field.name == "name" {
  //        if let crate::ds::Value::Str(s) = &field.kind {
  //          return s.clone();
  //        }
  //      }
  //    }
  //  }
  //  
  //  String::from("main")
  //}();

  //: qw.conf readed

  let farena = FileArena::new();

  build_cre(&info, &farena, std::path::Path::new(info.path).join("src").join("main.qw").to_str().unwrap().to_string())?;

  Ok(())
}
