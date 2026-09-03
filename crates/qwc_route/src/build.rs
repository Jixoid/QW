use std::{env, path::Path, time::{Duration, Instant}};
use owo_colors::OwoColorize;
use qwc_arena::Files;
use qwc_ast::{self as ast, Scope, StrInterner};
use qwc_diagnostic::Summary;
use qwc_front::Front;

use crate::{BuildVariant, Error, parse_conf};


pub struct BuildInfo<'a> {
  pub path: &'a str,
  pub variant: BuildVariant,
  pub verbose: u8,
  pub timings: bool,
  pub usages: bool,
  pub ast_dump: bool,
  pub check_only: bool,
}

/*
fn parse_deps(conf: &Value) -> Vec<(String, String)> {
  let mut deps = Vec::new();

  if let Value::Stc(stc) = conf {
    for (name, val) in stc {
      if name == "onerepo" {
        if let Value::Stc(repos) = &val {
          for (name, val) in repos {
            let dep_name = name.clone();
            match &val {
              Value::Str(path) => deps.push((dep_name, path.clone())),

              Value::Stc(repo_stc) => {
                for (name, val) in repo_stc {
                  if name == "path" {
                    if let Value::Str(path) = &val {
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


pub fn bduild_cre<'a>(name: String, info: &BuildInfo, fpath: String, deps: Vec<(String, String)>) -> Result<(), Error> {
  let mut ast_cre = ast::Krate::new();

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
    if far.is_loaded(&dep_fpath) { continue; }

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

  let mol = far.alloc(ast::Module::new(&fpath)?);
  
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

  /*
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
  */

  // Build Summary
  if info.timings {
    println!("{}{} {}ms", "build-time".yellow().bold(), ":".bright_black(), (front_time /*+hgen_time+mgen_time*/).as_millis());

    if info.verbose {
      println!("  {}{} {:?}", "front".blue().bold(), ":".bright_black(), front_time);
      //println!("  {}{} {:?}", "hgen".blue().bold(), ":".bright_black(), hgen_time);
      //println!("  {}{} {:?}", "mgen".blue().bold(), ":".bright_black(), mgen_time);
    }
  }

  // Usage
  if info.usages {
    println!("{}{} {}", "mem-usage".yellow().bold(), ":".bright_black(), humanize_size(ast_usage /*+hir_usage+mir_usage*/));

    if info.verbose {
      println!("  {}{} {}", "ast".blue().bold(), ":".bright_black(), humanize_size(ast_usage));
      //println!("  {}{} {}", "hir".blue().bold(), ":".bright_black(), humanize_size(hir_usage));
      //println!("  {}{} {}", "mir".blue().bold(), ":".bright_black(), humanize_size(mir_usage));
    }
  }

  Ok(())
}
*/


pub fn read_file(fpath: &Path, cre: &mut ast::Krate, sin: &mut StrInterner, far: &mut Files) -> Result<(ast::Item, Summary, Option<Scope>), Error> {
  let fi = {
    let fid = far.add(fpath);
    far.get(fid)
  };

  let (start_rng, mut sum) = Front::parse(cre, sin, far, fi);

  let (start, rng) = start_rng.unwrap();

  let submods_to_load = {
    let mut submods_to_load = vec![];
    
    for id in cre.extra_get(rng) {
      let id = ast::ItemId::new_from(id);
      let this: &ast::Item = cre.get(id);
      
      if let ast::ItemKind::ModuleUnloaded = this.kind {
        let name = sin.str(this.name.unwrap().sid()).to_string();
        let path = {
          let path1 = fpath.parent().unwrap().join(name.clone() + ".qw");
          let path2 = fpath.parent().unwrap().join(&name).join("mod.qw");
          
          match () {
            _ if path1.exists() => path1,
            _ if path2.exists() => path2,
            _ => return Err(Error::New | format!("could not find module file (`{:?}` or `{:?}`)", path1, path2)),
          }
        };
        
        submods_to_load.push((id, path));
      }
    }

    submods_to_load
  };

  for (id, path) in submods_to_load {
    let _ = read_file(&path, cre, sin, far).map(|(it, ssum, scp)| {
      scp.map(|scp| cre.attach(id, scp));
      
      let rng = if let ast::ItemKind::Krate(rng) = it.kind { rng } else { panic!() };
      
      let this: &mut ast::Item = cre.get_mut(id);
      
      this.kind = ast::ItemKind::ModuleFile(rng, it.pos.fid());

      sum += ssum;
    });
  }


  // Post
  let this = ast::Item{
    pos: start,
    vis: ast::Visibility::Inherited,
    name: None,
    kind: ast::ItemKind::Krate(rng)
  };

  let scp = Scope::new_with(cre, this);
  
  Ok((this, sum, scp))
}


pub fn build_ast_krate(fpath: &Path, info: &BuildInfo) -> Result<(ast::Krate, StrInterner, Files, Duration), Error> {
  let mut cre = ast::Krate::new();
  let mut far = Files::new();
  let mut sin = StrInterner::new();

  let legcurpath = env::current_dir()?;
  env::set_current_dir(fpath)?;

  let conf = parse_conf(fpath, &mut far)?;
  
  let entry_file = {
    let path1 = Path::new("src/main.qw");
    let path2 = Path::new("src/lib.qw");
    
    match () {
      _ if path1.exists() => path1,
      _ if path2.exists() => path2,
      
      _ => return Err(Error::New | format!("could not find entry file (`{:?}` or `{:?}`)", path1, path2))
    }
  };
  
  if info.verbose > 0 {
    eprintln!("{}{} {}", "Compiling".green().bold(), ":".bright_black(), conf.name);
  }
  
  let now = Instant::now();
  let (root, sum) = {
    let (root, sum, scp) = read_file(&entry_file, &mut cre, &mut sin, &mut far)?;

    let id = cre.push(root);
    scp.map(|scp| cre.attach(id, scp));
    
    (id, sum)
  };
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display(&far)) };
    
    eprintln!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "compilation stopped") }
  }

  cre.set_root(root);

  env::set_current_dir(legcurpath)?;
  
  Ok((cre, sin, far, time))
}


pub fn build(info: BuildInfo) -> Result<(), Error> {
  // PASS 1
  let (ast_cre, sin, far, front_time) = build_ast_krate(Path::new(info.path), &info)?;
  
  if info.ast_dump {
    eprint!("{}", ast::Dump{cre: &ast_cre, sin: &sin, far: &far});
  }
  
  // PASS 2
  // hir, hgen

  if info.check_only {
    return Ok(())
  }

  // PASS 3
  // mir, mgen


  // Extra Info
  if info.timings {
    eprintln!("{}: {:?}", "timings".yellow().bold(), (front_time));
    
    if info.verbose > 0 {
      eprintln!("  {}: {:?}", "front".yellow(), front_time);
    }
  }
  
  if info.usages {
    let (ast_used, ast_alloc) = (ast_cre.size_all_used(), ast_cre.size_all_alloc());

    eprintln!("{}: {} {} {}", "usages".yellow().bold(), humanize_size(ast_used), "/".bright_black(), humanize_size(ast_alloc));

    if info.verbose > 0 {
      eprintln!("  {}: {} {} {}", "ast".yellow(), humanize_size(ast_used), "/".bright_black(), humanize_size(ast_alloc));
      
      if info.verbose > 1 {
        eprintln!("   {}: {} {} {}", "type".cyan(), humanize_size(ast_cre.size_used::<ast::Type>()), "/".bright_black(), humanize_size(ast_cre.size_alloc::<ast::Type>()));
        eprintln!("   {}: {} {} {}", "expr".cyan(), humanize_size(ast_cre.size_used::<ast::Expr>()), "/".bright_black(), humanize_size(ast_cre.size_alloc::<ast::Expr>()));
        eprintln!("   {}: {} {} {}", "item".cyan(), humanize_size(ast_cre.size_used::<ast::Item>()), "/".bright_black(), humanize_size(ast_cre.size_alloc::<ast::Item>()));
        eprintln!("   {}: {} {} {}", "patt".cyan(), humanize_size(ast_cre.size_used::<ast::Patt>()), "/".bright_black(), humanize_size(ast_cre.size_alloc::<ast::Patt>()));
        eprintln!("   {}: {} {} {}", "thing".cyan(), humanize_size(ast_cre.size_used::<ast::Thing>()), "/".bright_black(), humanize_size(ast_cre.size_alloc::<ast::Thing>()));
        eprintln!("   {}: {} {} {}", "extra".cyan(), humanize_size(ast_cre.size_used::<ast::AnyId>()), "/".bright_black(), humanize_size(ast_cre.size_alloc::<ast::AnyId>()));
      }
    }
  }

  Ok(())
}

fn humanize_size(size: usize) -> String {
  let (val, set) = 'a: {
    let mut i = 0;
    let mut size = size as f64;

    loop {
      if size > 1024.0 { size /= 1024.0; i += 1; }
      else { break 'a (size, i); }
    }
  };

  let letter = match set { 0 => "b", 1 => "kb", 2 => "mb", 3 => "gb", 4 => "tb", _ => todo!() };

  format!("{:.2}{}", val, letter)
}
