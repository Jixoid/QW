use std::time::Instant;

use owo_colors::OwoColorize;
use crate::{control::module::{self, Module, ModuleKind}, front::Front, sema::Sema, sys::SysFile};


pub struct BuildInfo<'a> {
  pub path: &'a str,
  pub verbose: bool,
  pub timings: bool,
  pub ast_dump: bool,
  pub hir_dump: bool,
}


pub struct FileArena {
  files: std::cell::RefCell<Vec<Box<crate::control::module::ModuleFile>>>,
}

impl FileArena {
  pub fn new() -> Self { Self { files: std::cell::RefCell::new(Vec::new()) } }
  
  pub fn alloc(&self, file: crate::control::module::ModuleFile) -> &crate::control::module::ModuleFile {
    let b = Box::new(file);
    let ptr = &*b as *const crate::control::module::ModuleFile;
    self.files.borrow_mut().push(b);
    unsafe { &*ptr }
  }
}


pub struct ModInjection<'a> {
  pub sys: &'a SysFile<'a>,
  pub farena: &'a FileArena,
}


pub fn build_mod<'mi>(info: &BuildInfo, path: String, mi: &'mi ModInjection<'mi>) -> module::Result<()> {
  let (mut mol, mfd) = Module::new(path+"/src/main.qw", ModuleKind::Regular)?;
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

  let mangled_name = {
    let mut out = String::new();
    for part in mol.name.split(|c| c == ':' || c == '.' || c == '/') {
      if !part.is_empty() {
        out.push_str(&format!("{}{}", part.len(), part));
      }
    }
    if out.is_empty() {
      out = format!("{}{}", mol.name.len(), mol.name);
    }
    out
  };

  let out_dir = std::path::Path::new(info.path).join("build").join("mod");
  let _ = std::fs::create_dir_all(&out_dir);
  let out_file = out_dir.join(format!("{}.ll", mangled_name));
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
  let sys = SysFile::new()?;
  let farena = FileArena::new();

  let mi = ModInjection{sys: &sys, farena: &farena};

  build_mod(&info, info.path.to_string(), &mi)?;

  Ok(())
}
