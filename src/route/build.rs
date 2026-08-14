use std::time::Instant;

use owo_colors::OwoColorize;
use crate::{BuildVariant, ast::{self, Crate, Module}, error::{CompilerError, Result}, front::Front};


pub struct BuildInfo<'a> {
  pub path: &'a str,
  pub variant: BuildVariant,
  pub verbose: bool,
  pub timings: bool,
  pub usages: bool,
  pub ast_dump: bool,
  pub hir_dump: bool,
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



pub fn build_mod<'a>(info: &BuildInfo, cre: &mut ast::Crate<'a>, farena: &'a FileArena, fpath: String) -> Result<()> {
  let mol = farena.alloc(ast::Module::new(&fpath)?);
  
  let mut anys = Vec::new();
  
  let now: Instant;
  
  // Front
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "front".red().bold(), ":".bright_black(), mol.name) }
    
    let sum = Front::new(cre, &mol, farena)?.parse(&mut anys);
    
    /* time */ let front_time = now.elapsed();
    /* usag */ let ast_usage = cre.storage_size();
    
    // Summary
    for m in sum.msgs() { eprintln!("{}", m.display(farena)); }
    if sum.sumall() > 0 { eprintln!("{}", sum); }
    if sum.sumerr() > 0 { return Ok(()); }
    drop(sum);

    // Root
    let this = ast::Item{vis: ast::Visibility::Public, vari: ast::ItemVari::Module{name: mol.name.clone(), ctn: cre.new_extra(anys)}};
    cre.root_exec = cre.new_item(this);

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


  // HGen
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "hgen".red().bold(), ":".bright_black(), mol.name) }
    
    let target = crate::layout::Target::new_64bit();
    let is_debug = matches!(info.variant, crate::BuildVariant::Debug);
    let hir = crate::hgen::HGen::new(&mol, is_debug, &target).generate();
    
    /* time */ let hgen = now.elapsed();

    // HIR dump
    if info.hir_dump {
      println!("{}{} {}", "hir-dump".purple().bold(), ":".bright_black(), mol.name.white().bold());
      println!("{}", hir);
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
    println!("{}{} {:?}", "total-time".yellow().bold(), ":".bright_black(), (front_time/*+sema+hgen+cgen*/));
    
    if info.verbose {
      println!("  {}{} {:?}", "front".blue().bold(), ":".bright_black(), front_time);
      /*
      println!("  {}{}  {:?}", "sema".blue().bold(), ":".bright_black(), sema);
      println!("  {}{}  {:?}", "hgen".blue().bold(), ":".bright_black(), hgen);
      println!("  {}{}  {:?}", "cgen".blue().bold(), ":".bright_black(), cgen);
      */
    }
  }

  // Usage
  if info.usages {
    println!("{}{} {}", "mem-usage".yellow().bold(), ":".bright_black(), humanize_size(ast_usage));

    if info.verbose {
      println!("  {}{} {}", "ast".blue().bold(), ":".bright_black(), humanize_size(ast_usage));
    }
  }

  Ok(())
}


pub fn build(info: BuildInfo) -> Result<()> {
  let conf_path = std::path::Path::new(info.path).join("qw.conf");
  if !conf_path.exists() {
    return Err(CompilerError::Str("could not find `qw.conf`.".to_string()));
  }

  let mfd = Module::new(conf_path.to_str().unwrap_or(""))?;

  let conf = crate::ds::Value::load_file(&mfd).map_err(|e| CompilerError::Str(format!("{:?}", e)))?;

  let crate_name = || -> String {
    if let crate::ds::Value::Stc(stc) = conf {
      for field in &stc.subs {
        if field.name == "name" {
          if let crate::ds::Value::Str(s) = &field.kind {
            return s.clone();
          }
        }
      }
    }
    
    String::from("main")
  }();

  //: qw.conf readed

  let mut cre = Crate::new(crate_name);

  let farena = FileArena::new();

  build_mod(&info, &mut cre, &farena, std::path::Path::new(info.path).join("src").join("main.qw").to_str().unwrap().to_string())?;

  Ok(())
}
