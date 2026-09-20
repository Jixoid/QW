use std::{env, path::Path, time::{Duration, Instant}};
use owo_colors::OwoColorize;
use qwc_arena::Files;
use qwc_ast::{self as ast, Visitor};
use qwc_cgen::ICGen;
use qwc_cgen_llvm::CGenLLVM;
use qwc_hir::{self as hir};
use qwc_mir::{self as mir, Layout, LayoutInfo};
use qwc_diagnostic::{Label, Message, Summary, msg::*};
use qwc_parse::Parse;
use qwc_hir_gen::HGen;
use qwc_mir_gen::MGen;
use qwc_resolve::ScopeMap;
use qwc_string_interner::StrInterner;

use crate::{BuildVariant, DumpStage, Error, parse_conf};


pub struct BuildInfo<'a> {
  pub path: &'a str,
  pub variant: BuildVariant,
  pub verbose: u8,
  pub timings: bool,
  pub usages: bool,
  pub dump: Vec<DumpStage>,
  pub check_only: bool,
}


pub fn read_file(fpath: &Path, cre: &mut ast::Krate, sin: &mut StrInterner, far: &mut Files) -> Result<(ast::Item, Summary), Error> {
  let fi = {
    let fid = far.add(fpath);
    far.get(fid)
  };

  let (start, rng, mut sum) = Parse::parse(cre, sin, far, fi);

  let submods_to_load = {
    let mut submods_to_load = vec![];
    
    for id in cre.extra_get(rng) {
      let id = ast::ItemId::new_from(id);
      let it: &ast::Item = cre.get(id);
      
      if let ast::ItemKind::ModuleUnloaded = it.kind {
        let name = sin.str(it.name.unwrap().sid()).to_string();
        let path = {
          let path1 = fpath.parent().unwrap().join(name.clone() + ".qw");
          let path2 = fpath.parent().unwrap().join(&name).join("mod.qw");
          
          match () {
            _ if path1.exists() => path1,
            _ if path2.exists() => path2,
            _ => {
              sum.add(Message::error(COULD_NOT_FIND_MODULE_FILE, 
                Label::new(it.name.unwrap(), NOT_FOUND_IN_OR2.args(&[
                  path1.to_str().unwrap(),
                  path2.to_str().unwrap(),
                ])
              )));
              continue;
            }
          }
        };
        
        submods_to_load.push((id, path));
      }
    }

    submods_to_load
  };

  for (id, path) in submods_to_load {
    let _ = read_file(&path, cre, sin, far).map(|(it, ssum)| {
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

  Ok((this, sum))
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
    let (root, sum) = read_file(&entry_file, &mut cre, &mut sin, &mut far)?;

    let id = cre.push(root);
    
    (id, sum)
  };
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display(&far)) };
    
    eprint!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "") }
  }

  cre.set_root(root);

  env::set_current_dir(legcurpath)?;
  
  Ok((cre, sin, far, time))
}

pub fn build_ast_scope(ast_cre: &ast::Krate, sin: &StrInterner, far: &Files) -> Result<(ScopeMap, Duration), Error> {

  let now = Instant::now();
  let ret = ScopeMap::visit(&ast_cre, &sin, &far);
  let time = now.elapsed();

  match ret {
    Ok(v) => Ok((v, time)),
    Err(sum) => {
      for emsg in &sum { eprintln!("{}", emsg.display(&far)) };
    
      eprint!("{}", sum);

      Err(Error::New | "")
    }
  }
}

pub fn build_hir_krate(ast_cre: &ast::Krate, sin: &StrInterner, far: &Files, ast_scp: &ScopeMap) -> Result<(hir::Krate, Duration), Error> {
  let now = Instant::now();
  let (hir_cre, sum) = HGen::low(&ast_cre, sin, far, ast_scp);
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display(&far)) };
    
    eprint!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "") }
  }

  Ok((hir_cre.unwrap(), time))
}

pub fn build_mir_krate(hir_cre: &hir::Krate, sin: &StrInterner, far: &Files, layinfo: &LayoutInfo) -> Result<(mir::Krate, Duration), Error> {
  let now = Instant::now();
  let (mir_cre, sum) = MGen::low(&hir_cre, sin, layinfo);
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display(&far)) };
    
    eprint!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "") }
  }

  Ok((mir_cre.unwrap(), time))
}

pub fn build_cgen(mir_cre: &mir::Krate) -> Result<Duration, Error> {
  let now = Instant::now();
  CGenLLVM::generate(mir_cre);
  let time = now.elapsed();

  Ok(time)
}



pub fn build(info: BuildInfo) -> Result<(), Error> {
  // PASS 1
  if info.verbose > 0 { eprintln!("{}", "parse".red().bold()) }
  
  let (ast_cre, sin, far, parse_time) = build_ast_krate(Path::new(info.path), &info)?;
  
  if info.dump.contains(&DumpStage::Ast) { eprintln!("{}", ast::Dump{cre: &ast_cre, sin: &sin, far: &far}) }
  
  
  // PASS 1 + ScopeMap
  if info.verbose > 0 { eprintln!("{}", "scope".red().bold()) }
  
  let (ast_scp, scope_time) = build_ast_scope(&ast_cre, &sin, &far)?;
  
  if info.dump.contains(&DumpStage::Scope) { eprintln!("{}", qwc_resolve::dupm_scp::Dump{scp: &ast_scp, sin: &sin, root: ast_cre.root().unwrap().to_any()}) }
  
  
  // PASS 2
  if info.verbose > 0 { eprintln!("{}", "hgen".red().bold()) }
  
  let (hir_cre, hgen_time) = build_hir_krate(&ast_cre, &sin, &far, &ast_scp)?;
  
  if info.check_only {
    return Ok(())
  }

  
  // Target
  let layinfo = LayoutInfo{
    i8_lay:   Layout::new(8,   8,  mir::LayoutBy::SYS),
    i16_lay:  Layout::new(16,  16, mir::LayoutBy::SYS),
    i32_lay:  Layout::new(32,  32, mir::LayoutBy::SYS),
    i64_lay:  Layout::new(64,  64, mir::LayoutBy::SYS),
    i128_lay: Layout::new(128, 64, mir::LayoutBy::SYS),

    bf16_lay: Layout::new(16,  16, mir::LayoutBy::SYS),
    f16_lay:  Layout::new(16,  16, mir::LayoutBy::SYS),
    f32_lay:  Layout::new(32,  32, mir::LayoutBy::SYS),
    f64_lay:  Layout::new(64,  64, mir::LayoutBy::SYS),
    f128_lay: Layout::new(128, 64, mir::LayoutBy::SYS),

    ptr_size: Layout::new(64, 64, mir::LayoutBy::SYS),
  };
  
  
  // Pass 3
  if info.verbose > 0 { eprintln!("{}", "mgen".red().bold()) }
  
  let (mir_cre, mgen_time) = build_mir_krate(&hir_cre, &sin, &far, &layinfo)?;
  
  if info.dump.contains(&DumpStage::Mir) { eprintln!("{}", mir::Dump{cre: &mir_cre}) }
  
  
  // Pass 4
  if info.verbose > 0 { eprintln!("{}", "cgen".red().bold()) }

  let cgen_time = build_cgen(&mir_cre)?;


  // Extra Info
  if info.timings {
    eprintln!("{}: {:?}", "timings".yellow().bold(), (parse_time + scope_time + hgen_time + mgen_time + cgen_time));
    
    if info.verbose > 0 {
      eprintln!("  {}: {:?}", "parse".yellow(), parse_time);
      eprintln!("  {}: {:?}", "scope".yellow(), scope_time);
      eprintln!("  {}:  {:?}", "hgen".yellow(), hgen_time);
      eprintln!("  {}:  {:?}", "mgen".yellow(), mgen_time);
      eprintln!("  {}:  {:?}", "cgen".yellow(), cgen_time);
    }
  }
  
  if info.usages {
    let (ast_used, ast_alloc) = (ast_cre.size_all_used(), ast_cre.size_all_alloc());
    let (hir_used, hir_alloc) = (hir_cre.size_all_used(), hir_cre.size_all_alloc());
    let (mir_used, mir_alloc) = (mir_cre.size_all_used(), mir_cre.size_all_alloc());

    eprintln!("{}: {} {} {}", "usages".yellow().bold(), humanize_size(ast_used + hir_used + mir_used), "/".bright_black(), humanize_size(ast_alloc + hir_alloc + mir_alloc));

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
      
      eprintln!("  {}: {} {} {}", "hir".yellow(), humanize_size(hir_used), "/".bright_black(), humanize_size(hir_alloc));
      if info.verbose > 1 {
        eprintln!("   {}: {} {} {}", "type".cyan(), humanize_size(hir_cre.size_used::<hir::Type>()), "/".bright_black(), humanize_size(hir_cre.size_alloc::<hir::Type>()));
        eprintln!("   {}: {} {} {}", "expr".cyan(), humanize_size(hir_cre.size_used::<hir::Expr>()), "/".bright_black(), humanize_size(hir_cre.size_alloc::<hir::Expr>()));
        eprintln!("   {}: {} {} {}", "item".cyan(), humanize_size(hir_cre.size_used::<hir::Item>()), "/".bright_black(), humanize_size(hir_cre.size_alloc::<hir::Item>()));
        eprintln!("   {}: {} {} {}", "extra".cyan(), humanize_size(hir_cre.size_used::<hir::AnyId>()), "/".bright_black(), humanize_size(hir_cre.size_alloc::<hir::AnyId>()));
      }

      eprintln!("  {}: {} {} {}", "mir".yellow(), humanize_size(mir_used), "/".bright_black(), humanize_size(mir_alloc));
      if info.verbose > 1 {
        eprintln!("   {}: {} {} {}", "type".cyan(), humanize_size(mir_cre.size_used::<mir::Type>()), "/".bright_black(), humanize_size(mir_cre.size_alloc::<mir::Type>()));
        eprintln!("   {}: {} {} {}", "extra".cyan(), humanize_size(mir_cre.size_used::<mir::AnyId>()), "/".bright_black(), humanize_size(mir_cre.size_alloc::<mir::AnyId>()));
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
