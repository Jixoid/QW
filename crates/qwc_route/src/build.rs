use std::{env, fs, io::Write, path::Path, time::{Duration, Instant}};
use owo_colors::OwoColorize;
use qwc_arena::Files;
use qwc_ast as ast;
use qwc_cgen::ICGen;
use qwc_cgen_llvm::CGenLLVM;
use qwc_hir as hir;
use qwc_mir as mir;
use qwc_diagnostic::{Label, Message, Summary, msg::*};
use qwc_parse::Parse;
use qwc_hir_gen::HGen;
use qwc_mir_gen::MGen;
use qwc_resolve::{ExportMap, Imod, ScopeCollector, ScopeMap};
use qwc_string_interner::StrInterner;
use qwc_unit::Unit;

use crate::{BuildVariant, DumpStage, Error, parse_conf};


pub struct BuildInfo<'a> {
  pub path: &'a Path,
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
  let abs_fpath = fpath.canonicalize().unwrap_or_else(|_| fpath.to_path_buf());
  env::set_current_dir(&abs_fpath)?;

  let conf = parse_conf(&abs_fpath, &mut far)?;
  
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

pub fn build_ast_scope(ast_cre: &ast::Krate, sin: &StrInterner, far: &Files, imods: &[Imod<'_>]) -> Result<(ScopeMap, Duration), Error> {
  let now = Instant::now();
  let ret = ScopeCollector::collect(ast_cre, sin, imods);
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

pub fn build_hir_krate(ast_cre: &ast::Krate, sin: &StrInterner, far: &Files, ast_scp: &ScopeMap, imods: &[hir::CID], ideps: &mut hir::Deps) -> Result<(hir::CID, Duration), Error> {
  let now = Instant::now();
  let (hir_cid, sum) = HGen::low(&ast_cre, sin, far, ast_scp, imods, ideps);
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display(&far)) };
    
    eprint!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "") }
  }

  Ok((hir_cid.unwrap(), time))
}

pub fn build_hir_export(hir_cre: &hir::Krate, far: &Files) -> Result<(ExportMap, Duration), Error> {
  use hir::Visitor;

  let now = Instant::now();
  let ret = ExportMap::visit(hir_cre);
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

pub fn build_mir_krate(hir_cre: &hir::Krate, sin: &StrInterner, far: &Files, layinfo: &mir::LayoutInfo) -> Result<(mir::Krate, Duration), Error> {
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

pub fn build_cgen(mir_cre: &mir::Krate, fpath: &Path) -> Result<Duration, Error> {
  let now = Instant::now();
  
  CGenLLVM::generate(mir_cre, fpath).map_err(|err| Error::Str(err))?;

  let time = now.elapsed();

  Ok(time)
}



pub fn build(info: BuildInfo) -> Result<(), Error> {
  // Layout
  let hir_layinfo = hir::LayoutInfo{
    bool_lay: hir::Layout::new_static(hir::LayoutBy::SYS),

    i8_lay:   hir::Layout::new_static(hir::LayoutBy::SYS),
    i16_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
    i32_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
    i64_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
    i128_lay: hir::Layout::new_static(hir::LayoutBy::SYS),

    bf16_lay: hir::Layout::new_static(hir::LayoutBy::SYS),
    f16_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
    f32_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
    f64_lay:  hir::Layout::new_static(hir::LayoutBy::SYS),
    f128_lay: hir::Layout::new_static(hir::LayoutBy::SYS),

    ptr_size: hir::Layout::new_static(hir::LayoutBy::SYS),
  };

  let mir_layinfo = mir::LayoutInfo{
    i8_lay:   mir::Layout::new_sst(1,  1,  mir::LayoutBy::SYS),
    i16_lay:  mir::Layout::new_sst(2,  2,  mir::LayoutBy::SYS),
    i32_lay:  mir::Layout::new_sst(4,  4,  mir::LayoutBy::SYS),
    i64_lay:  mir::Layout::new_sst(8,  8,  mir::LayoutBy::SYS),
    i128_lay: mir::Layout::new_sst(16, 16, mir::LayoutBy::SYS),

    bf16_lay: mir::Layout::new_sst(1,  1,  mir::LayoutBy::SYS),
    f16_lay:  mir::Layout::new_sst(2,  2,  mir::LayoutBy::SYS),
    f32_lay:  mir::Layout::new_sst(4,  4,  mir::LayoutBy::SYS),
    f64_lay:  mir::Layout::new_sst(8,  8,  mir::LayoutBy::SYS),
    f128_lay: mir::Layout::new_sst(16, 16, mir::LayoutBy::SYS),

    ptr_size: mir::Layout::new_sst(8, 8, mir::LayoutBy::SYS),
  };


  // PASS 1 (parse)
  if info.verbose > 0 { eprintln!("{}", "PASS 1 (parse)".red().bold()) }
  
  let (ast_cre, mut sin, far, time_pass1_parse) = build_ast_krate(info.path, &info)?;
  
  if info.dump.contains(&DumpStage::Ast) { eprintln!("{}", ast::Dump{cre: &ast_cre, sin: &sin, far: &far}) }


  // Core & Imods
  let mut deps = hir::Deps::new();
  let core_cid = deps.get_next_id();
  let (core_cre, core_exp) = qwc_intrinsic::new_core(core_cid, &mut sin, &hir_layinfo);
  deps.add(core_cre);
  let core_name = sin.sid("core");
  let imods = [Imod::new(core_name, &core_exp)];
  let imod_cids = [core_cid];

  
  // PASS 1 (scope)
  if info.verbose > 0 { eprintln!("{}", "PASS 1 (scope)".red().bold()) }
  
  let (scp, time_pass1_scope) = build_ast_scope(&ast_cre, &sin, &far, &imods)?;
  
  if info.dump.contains(&DumpStage::Scope) { eprintln!("{}", qwc_resolve::dupm_scp::Dump{scp: &scp, sin: &sin, root: ast_cre.root().unwrap().to_any()}) }
  
  
  // PASS 2 (hgen)
  if info.verbose > 0 { eprintln!("{}", "PASS 2 (hgen)".red().bold()) }
  
  let (hir_cid, time_pass2_hgen) = build_hir_krate(&ast_cre, &sin, &far, &scp, &imod_cids, &mut deps)?;
  let hir_cre = deps.get(hir_cid);
  
  if info.dump.contains(&DumpStage::Hir) { eprintln!("{}", hir::Dump{cre: hir_cre, sin: &sin}) }
  
  if info.check_only { return Ok(()) }


  // DROP scp
  drop(scp);
  
  
  // PASS 2 (export)
  if info.verbose > 0 { eprintln!("{}", "PASS 2 (export)".red().bold()) }
  
  let (exp, time_pass2_export) = build_hir_export(hir_cre, &far)?;
  
  if info.dump.contains(&DumpStage::Export) { eprintln!("{}", qwc_resolve::dump_exp::Dump{exp: &exp, cre: hir_cre, sin: &sin, root: hir_cre.root().unwrap().to_any()}) }


  // QWU save
  let mut qwu = fs::File::create(info.path.join("build").join("out.qwu"))?;

  Unit::serialize(&mut qwu, hir_cre, &exp)?;

  qwu.flush()?;

  
  // Pass 3 (mgen)
  if info.verbose > 0 { eprintln!("{}", "PASS 3 (mgen)".red().bold()) }
  
  let (mir_cre, time_pass3_mgen) = build_mir_krate(hir_cre, &sin, &far, &mir_layinfo)?;
  
  if info.dump.contains(&DumpStage::Mir) { eprintln!("{}", mir::Dump{cre: &mir_cre}) }
  
  
  // Pass 4 (cgen)
  if info.verbose > 0 { eprintln!("{}", "PASS 4 (cgen)".red().bold()) }
  
  if !info.path.join("build").exists() {
    fs::create_dir(info.path.join("build"))?;
  }
  
  let time_pass4_cgen = build_cgen(&mir_cre, &info.path.join("build").join("out.ll"))?;
  
  if info.dump.contains(&DumpStage::Lir) { eprintln!("{}", fs::read_to_string(info.path.join("build").join("out.ll"))?) }


  // Timings
  if info.timings {
    let pass1 = time_pass1_parse + time_pass1_scope;
    let pass2 = time_pass2_hgen + time_pass2_export;
    let pass3 = time_pass3_mgen;
    let pass4 = time_pass4_cgen;

    eprintln!("{}{} {:?}", "timings".yellow().bold(), ":".bright_black(), (pass1 + pass2 + pass3 + pass4));
    
    if info.verbose > 0 {
      eprintln!("  {}{} {:?}", "pass 1".yellow(), ":".bright_black(), pass1);
      if info.verbose > 1 {
        eprintln!("    {}{} {:?}", "parse".cyan(), ":".bright_black(), time_pass1_parse);
        eprintln!("    {}{} {:?}", "scope".cyan(), ":".bright_black(), time_pass1_scope);
      }
      
      eprintln!("  {}{} {:?}", "pass 2".yellow(), ":".bright_black(), pass2);
      if info.verbose > 1 {
        eprintln!("    {}{} {:?}", "hgen".cyan(), ":".bright_black(), time_pass2_hgen);
        eprintln!("    {}{} {:?}", "export".cyan(), ":".bright_black(), time_pass2_export);
      }
      
      eprintln!("  {}{} {:?}", "pass 3".yellow(), ":".bright_black(), pass3);
      if info.verbose > 1 {
        eprintln!("    {}{} {:?}", "mgen".cyan(), ":".bright_black(), time_pass3_mgen);
      }
      
      eprintln!("  {}{} {:?}", "pass 4".yellow(), ":".bright_black(), pass4);
      if info.verbose > 1 {
        eprintln!("    {}{} {:?}", "cgen".cyan(), ":".bright_black(), time_pass4_cgen);
      }
      
      eprintln!()
    }
  }
  

  // Usages
  if info.usages {
    let (ast_used, ast_alloc) = (ast_cre.size_all_used(), ast_cre.size_all_alloc());
    let (hir_used, hir_alloc) = (hir_cre.size_all_used(), hir_cre.size_all_alloc());
    let (mir_used, mir_alloc) = (mir_cre.size_all_used(), mir_cre.size_all_alloc());

    eprintln!("{}: {} {} {}", "usages".yellow().bold(), humanize_size(ast_used + hir_used + mir_used), "/".bright_black(), humanize_size(ast_alloc + hir_alloc + mir_alloc));

    if info.verbose > 0 {
      eprintln!("  {}: {} {} {}", "ast".yellow(), humanize_size(ast_used), "/".bright_black(), humanize_size(ast_alloc));
      if info.verbose > 1 {
        eprintln!("    {}: {}", "type".cyan(), humanize_size(ast_cre.size_used::<ast::Type>()));
        eprintln!("    {}: {}", "expr".cyan(), humanize_size(ast_cre.size_used::<ast::Expr>()));
        eprintln!("    {}: {}", "item".cyan(), humanize_size(ast_cre.size_used::<ast::Item>()));
        eprintln!("    {}: {}", "patt".cyan(), humanize_size(ast_cre.size_used::<ast::Patt>()));
        eprintln!("    {}: {}", "thing".cyan(), humanize_size(ast_cre.size_used::<ast::Thing>()));
        eprintln!("    {}: {}", "extra".cyan(), humanize_size(ast_cre.size_used::<ast::AnyId>()));
      }
      
      eprintln!("  {}: {} {} {}", "hir".yellow(), humanize_size(hir_used), "/".bright_black(), humanize_size(hir_alloc));
      if info.verbose > 1 {
        eprintln!("    {}: {}", "type".cyan(), humanize_size(hir_cre.size_used::<hir::Type>()));
        eprintln!("    {}: {}", "expr".cyan(), humanize_size(hir_cre.size_used::<hir::Expr>()));
        eprintln!("    {}: {}", "item".cyan(), humanize_size(hir_cre.size_used::<hir::Item>()));
        eprintln!("    {}: {}", "extra".cyan(), humanize_size(hir_cre.size_used::<hir::AnyId>()));
      }

      eprintln!("  {}: {} {} {}", "mir".yellow(), humanize_size(mir_used), "/".bright_black(), humanize_size(mir_alloc));
      if info.verbose > 1 {
        eprintln!("    {}: {}", "type".cyan(), humanize_size(mir_cre.size_used::<mir::Type>()));
        eprintln!("    {}: {}", "symb".cyan(), humanize_size(mir_cre.size_used::<mir::Symbol>()));
        eprintln!("    {}: {}", "blok".cyan(), humanize_size(mir_cre.size_used::<mir::Block>()));
        eprintln!("    {}: {}", "inst".cyan(), humanize_size(mir_cre.size_used::<mir::Inst>()));
        eprintln!("    {}: {}", "extra".cyan(), humanize_size(mir_cre.size_used::<mir::AnyId>()));
      }

      eprintln!()
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
