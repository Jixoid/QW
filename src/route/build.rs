use std::time::Instant;

use owo_colors::OwoColorize;
use crate::{control::module::{self, Module, ModuleKind}, front::Front, sema::Sema, sys::SysFile};


pub struct BuildInfo<'a> {
  pub path: &'a str,
  pub verbose: bool,
  pub timings: bool,
  pub ast_dump: bool,
  pub hir_dump: bool,
  pub check_only: bool,
}


pub struct FileArena {
  files: std::cell::RefCell<Vec<Box<crate::control::module::ModuleFile>>>,
  loaded_paths: std::cell::RefCell<std::collections::HashSet<String>>,
}

impl FileArena {
  pub fn new() -> Self { Self { files: std::cell::RefCell::new(Vec::new()), loaded_paths: std::cell::RefCell::new(std::collections::HashSet::new()) } }
  
  pub fn alloc(&self, file: crate::control::module::ModuleFile) -> &crate::control::module::ModuleFile {
    let b = Box::new(file);
    let ptr = &*b as *const crate::control::module::ModuleFile;
    self.loaded_paths.borrow_mut().insert(b.fpath.clone());
    self.files.borrow_mut().push(b);
    unsafe { &*ptr }
  }

  pub fn is_loaded(&self, fpath: &str) -> bool {
    self.loaded_paths.borrow().contains(fpath)
  }
}


pub struct ModInjection<'a> {
  pub sys: &'a SysFile<'a>,
  pub farena: &'a FileArena,
}


pub fn build_mod<'mi>(info: &BuildInfo, path: String, project_name: String, mi: &'mi ModInjection<'mi>) -> module::Result<()> {
  let (mut mol, mfd) = Module::new(path+"/src/main.qw", ModuleKind::Regular)?;
  mol.name = project_name.clone();
  if let Some(root_decl) = mol.list_decl.get_mut(0) {
    root_decl.name = crate::ast::DeclName::Name(project_name.clone());
  }
  mol.add_dep(&mi.sys.mol);

  let mut now: Instant;
  
  
  // Front
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "front".red().bold(), ":".bright_black(), mol.name) }
    
    let sum = Front::new(&mut mol, &mfd)?.parse(mi);
    
    /* time */ let front = now.elapsed();
    
    // Summary
    for m in sum.msgs() { println!("{}", m); }
    if sum.sumall() > 0 { println!("{}", sum); }
    if sum.sumerr() > 0 { return Ok(()); }
    drop(sum);


  // Sema
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "sema".red().bold(), ":".bright_black(), mol.name) }
    
    let sum = Sema::new(&mut mol).check();
    
    /* time */ let sema = now.elapsed();

    // Summary
    for m in sum.msgs() { println!("{}", m); }
    if sum.sumall() > 0 { println!("{}", sum); }
    if sum.sumerr() > 0 { return Ok(()); }
    drop(sum);
  
    // AST dump
    if info.ast_dump {
      println!("{}{} {}", "ast-dump".purple().bold(), ":".bright_black(), mol.name.white().bold());
      println!("{}", mol);
    }

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
    
    let hir = crate::hgen::HGen::new(&mol).generate();
    
    /* time */ let hgen = now.elapsed();

    // HIR dump
    if info.hir_dump {
      println!("{}{} {}", "hir-dump".purple().bold(), ":".bright_black(), mol.name.white().bold());
      println!("{}", hir);
    }


  // CGen
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "cgen".red().bold(), ":".bright_black(), mol.name) }
    
    let target = crate::layout::Target::new_64bit();
    let bin = crate::cgen::CGen::new(&hir, crate::basic_cgen::BasicCGen::new(&target)).generate();
  
    /* time */ let cgen = now.elapsed();

  let out_dir = std::path::Path::new(info.path).join("build");
  let _ = std::fs::create_dir_all(&out_dir);
  let out_file = out_dir.join("out.ll");
  let _ = std::fs::write(&out_file, &bin);


  // Time
  if info.timings {
    println!("{}{} {:?}", "total-time".yellow().bold(), ":".bright_black(), (front+sema+hgen+cgen));
    
    if info.verbose {
      println!("  {}{} {:?}", "front".blue().bold(), ":".bright_black(), front);
      println!("  {}{}  {:?}", "sema".blue().bold(), ":".bright_black(), sema);
      println!("  {}{}  {:?}", "hgen".blue().bold(), ":".bright_black(), hgen);
      println!("  {}{}  {:?}", "cgen".blue().bold(), ":".bright_black(), cgen);
    }
  }

  Ok(())
}


pub fn build(info: BuildInfo) -> module::Result<()> {
  let conf_path = std::path::Path::new(info.path).join("qw.conf");
  if !conf_path.exists() {
    return Err(crate::control::module::CompilerError::Str("could not find `qw.conf`.".to_string()));
  }

  let mmap = std::fs::read_to_string(&conf_path).map_err(|e| crate::control::module::CompilerError::Str(e.to_string()))?;
  let mfd = crate::control::module::ModuleFile {
    fpath: conf_path.to_str().unwrap_or("").to_string(),
    mmap,
    kind: crate::control::module::ModuleKind::Regular,
  };
  
  let conf = crate::ds::Value::load_file(&mfd).map_err(|e| crate::control::module::CompilerError::Str(format!("{:?}", e)))?;

  let mut project_name = String::from("main");
  if let crate::ds::Value::Stc(stc) = conf {
    for field in &stc.subs {
      if field.name == "name" {
        if let crate::ds::Value::Str(s) = &field.kind {
          project_name = s.clone();
        }
      }
    }
  }

  let sys = SysFile::new()?;
  let farena = FileArena::new();

  let mi = ModInjection{sys: &sys, farena: &farena};

  build_mod(&info, info.path.to_string(), project_name, &mi)?;

  Ok(())
}
