use std::time::Instant;

use owo_colors::OwoColorize;
use crate::{BuildVariant, ast, error::{CompilerError, Result}, front::Front, hgen::HGen, mgen::mgen::MGen};


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



fn parse_deps(conf: &crate::ds::Value) -> Vec<(String, String)> {
  let mut deps = Vec::new();

  if let crate::ds::Value::Stc(stc) = conf {
    for field in &stc.subs {
      if field.name == "onerepo" {
        if let crate::ds::Value::Stc(repos) = &field.kind {
          for repo in &repos.subs {
            let dep_name = repo.name.clone();
            match &repo.kind {
              crate::ds::Value::Str(path) => {
                deps.push((dep_name, path.clone()));
              }
              crate::ds::Value::Stc(repo_stc) => {
                for repo_field in &repo_stc.subs {
                  if repo_field.name == "path" {
                    if let crate::ds::Value::Str(path) = &repo_field.kind {
                      deps.push((dep_name.clone(), path.clone()));
                    }
                  }
                }
              }
              _ => {}
            }
          }
        }
      }
    }
  }

  deps
}


pub fn build_cre<'a>(name: String, info: &BuildInfo, farena: &'a FileArena, fpath: String, deps: Vec<(String, String)>) -> Result<()> {
  let mut ast_cre = ast::Crate::new();

  let mut anys = vec![];
  let mut dep_errors = 0;

  // Load dependencies into root crate
  for (dep_name, dep_path) in deps {
    let dep_src = std::path::Path::new(&dep_path).join("src");
    let dep_entry = if dep_src.join("lib.qw").exists() {
      dep_src.join("lib.qw")
    } else if dep_src.join("main.qw").exists() {
      dep_src.join("main.qw")
    } else {
      continue;
    };

    let dep_fpath = dep_entry.to_str().unwrap_or("").to_string();
    if farena.is_loaded(&dep_fpath) { continue; }

    let dep_mol = farena.alloc(ast::Module::new(&dep_fpath)?);
    let mut dep_anys = vec![];

    if info.verbose { println!("{}{} {} ({})", "front".red().bold(), ":".bright_black(), dep_name, dep_mol.name); }
    let dep_sum = Front::new(&mut ast_cre, dep_mol, farena)?.parse(&mut dep_anys);

    for m in dep_sum.msgs() { eprintln!("{}", m.display(farena)); }
    dep_errors += dep_sum.sumerr();

    let _ = ast_cre.get_str(&dep_name);
    let dep_scp = ast::Scope::from_anys(&ast_cre, &dep_anys);
    let dep_item = ast::Item {
      vis: ast::Visibility::Public,
      vari: ast::ItemVari::Module {
        name: dep_name,
        ctn: ast_cre.new_extra(dep_anys),
      },
    };
    let dep_id = ast_cre.new_item(dep_item);
    ast_cre.attach_scp(dep_id.to_any(), dep_scp);
    anys.push(dep_id.to_any());
  }

  if dep_errors > 0 { return Ok(()); }

  let mol = farena.alloc(ast::Module::new(&fpath)?);
  
  let mut now: Instant;
  
  // Front
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "front".red().bold(), ":".bright_black(), mol.name) }
    
    let sum = Front::new(&mut ast_cre, &mol, farena)?.parse(&mut anys);

    let _ = ast_cre.get_str(&name);
    let root_scp = ast::Scope::from_anys(&ast_cre, &anys);
    let root = ast::Item{vis: ast::Visibility::Public, vari: ast::ItemVari::Module{ name, ctn: ast_cre.new_extra(anys) }};
    let root_id = ast_cre.new_item(root);
    ast_cre.root = root_id;
    ast_cre.attach_scp(root_id.to_any(), root_scp);

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
    
    let hir_cre = match HGen::lower(&ast_cre, farena) {
      Ok(a) => a,
      Err(e) => return Err(CompilerError::Str( format!("{}", e.display(farena)) )),
    };
    
    /* time */ let hgen_time = now.elapsed();
    /* usag */ let hir_usage = hir_cre.storage_size();

  
  // MGen
    /* time */ now = Instant::now();
    /* verb */ if info.verbose { println!("{}{} {}", "mgen".red().bold(), ":".bright_black(), mol.name) }

    let mir_cre = match MGen::lower(&hir_cre) {
      Ok(a) => a,
      Err(e) => return Err(CompilerError::Str( format!("{}", e.display(farena)) )),
    };
    
    /* time */ let mgen_time = now.elapsed();
    /* usag */ let mir_usage = mir_cre.storage_size();


  // Build Summary
  if info.timings {
    println!("{}{} {}ms", "build-time".yellow().bold(), ":".bright_black(), (front_time+hgen_time+mgen_time).as_millis());

    if info.verbose {
      println!("  {}{} {:?}", "front".blue().bold(), ":".bright_black(), front_time);
      println!("  {}{} {:?}", "hgen".blue().bold(), ":".bright_black(), hgen_time);
      println!("  {}{} {:?}", "mgen".blue().bold(), ":".bright_black(), mgen_time);
    }
  }

  // Usage
  if info.usages {
    println!("{}{} {}", "mem-usage".yellow().bold(), ":".bright_black(), humanize_size(ast_usage+hir_usage+mir_usage));

    if info.verbose {
      println!("  {}{} {}", "ast".blue().bold(), ":".bright_black(), humanize_size(ast_usage));
      println!("  {}{} {}", "hir".blue().bold(), ":".bright_black(), humanize_size(hir_usage));
      println!("  {}{} {}", "mir".blue().bold(), ":".bright_black(), humanize_size(mir_usage));
    }
  }

  Ok(())
}


pub fn build(info: BuildInfo) -> Result<()> {
  let conf_path = std::path::Path::new(info.path).join("qw.conf");
  if !conf_path.exists() {
    return Err(CompilerError::Str("could not find `qw.conf`.".to_string()));
  }

  let mfd = ast::Module::new(conf_path.to_str().unwrap_or(""))?;

  let conf = crate::ds::Value::load_file(&mfd).map_err(|e| CompilerError::Str(format!("{:?}", e)))?;

  let crate_name = || -> Result<String> {
    if let crate::ds::Value::Stc(stc) = &conf {
      for field in &stc.subs {
        if field.name == "name" {
          if let crate::ds::Value::Str(s) = &field.kind {
            return Ok(s.clone());
          }
        }
      }
    }

    return Err(CompilerError::Str("name, not finded in `qw.conf`".to_string()));
  }()?;

  let deps = parse_deps(&conf);

  //: qw.conf readed

  let farena = FileArena::new();

  let src_dir = std::path::Path::new(info.path).join("src");
  let entry_file = if src_dir.join("main.qw").exists() {
    src_dir.join("main.qw")
  } else if src_dir.join("lib.qw").exists() {
    src_dir.join("lib.qw")
  } else {
    return Err(CompilerError::Str("could not find entry file (`src/main.qw` or `src/lib.qw`).".to_string()));
  };

  build_cre(crate_name, &info, &farena, entry_file.to_str().unwrap().to_string(), deps)?;

  Ok(())
}
