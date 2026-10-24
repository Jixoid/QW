/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use std::{env, fs, io::{self, Write}, path::{Path, PathBuf}, process::Command, time::{Duration, Instant}};
use owo_colors::OwoColorize;
use qwc_arena::Files;
use qwc_ast as ast;
use qwc_cgen::{self as cgen, ICGen, Optimization, OutKind};
use qwc_hir as hir;
use qwc_mir as mir;
use qwc_diagnostic::{Label, Message, Summary, msg::*};
use qwc_parse::Parse;
use qwc_hir_gen::HGen;
use qwc_mir_gen::MGen;
use qwc_resolve::{ExportMap, Imod, ImplFor, ScopeCollector, ScopeMap};
use qwc_string_interner::StrInterner;
use qwc_unit::Unit;

use crate::{BuildStartRoutine, BuildVariant, CodeModel, DumpStage, Error, OptLevel, RelocMode};


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BuildKind { Exec, Lib }

#[derive(Clone, Debug)]
pub struct DepArtifact {
  pub name: String,
  pub qwu_path: PathBuf,
  pub obj_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct CompileInfo<'a> {
  pub path: &'a Path,
  pub outdir: PathBuf,
  pub opt_level: OptLevel,
  pub variant: BuildVariant,
  pub start_routine: BuildStartRoutine,
  pub triple: Option<String>,
  pub rtl: Option<String>,
  pub reloc: RelocMode,
  pub mcmodel: CodeModel,
  pub verbose: u8,
  pub timings: bool,
  pub usages: bool,
  pub dump: Vec<DumpStage>,
  pub check_only: bool,
  pub execute: bool,
  pub deps: Vec<DepArtifact>,
}


pub fn read_file(fpath: &Path, cre: &mut ast::Krate, sin: &mut StrInterner, far: &mut Files) -> Result<(ast::Item, Summary), Error> {
  let fi = {
    let fid = far.add(fpath);
    far.get(fid)
  };

  let stem = match fpath.file_stem().and_then(|s| s.to_str()) {
    Some("mod") => fpath.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("mod"),
    Some(stem) => stem,
    None => "main",
  };

  let sid = sin.sid(stem);

  let (start, rng, mut sum) = Parse::parse(cre, sid, sin, far, fi);

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
      let ast::ItemKind::Krate(rng) = it.kind else { unreachable!() };
      
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
    kind: ast::ItemKind::Krate(rng),
    path: cre.push(ast::DefPath::Root(sid))
  };

  Ok((this, sum))
}


pub fn build_ast_krate(fpath: &Path, far: &mut Files) -> Result<(ast::Krate, StrInterner, BuildKind, Duration), Error> {
  let mut cre = ast::Krate::new();
  let mut sin = StrInterner::new();

  let main_cwd = env::current_dir()?;

  let (raw_entry, bldkind) = if fpath.is_file() {
    (fpath.to_path_buf(), BuildKind::Exec)
  } else {
    let path1 = fpath.join("src/main.qw");
    let path2 = fpath.join("src/lib.qw");

    match () {
      _ if path1.exists() => (path1, BuildKind::Exec),
      _ if path2.exists() => (path2, BuildKind::Lib),
      _ => {
        return Err(Error::New | format!("could not find entry file (`{:?}` or `{:?}`)", path1, path2));
      }
    }
  };

  // Convert entry file path to be relative to main execution directory (main_cwd)
  let entry_file = if let Ok(abs) = raw_entry.canonicalize() {
    if let Ok(rel) = abs.strip_prefix(&main_cwd) {
      rel.to_path_buf()
    } else {
      raw_entry
    }
  } else if let Ok(rel) = raw_entry.strip_prefix(&main_cwd) {
    rel.to_path_buf()
  } else {
    raw_entry
  };

  let now = Instant::now();
  let (root, sum) = {
    let (root, sum) = read_file(&entry_file, &mut cre, &mut sin, far)?;

    let id = cre.push(root);

    (id, sum)
  };
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display_human(&far)) };

    eprint!("{}", sum);

    if sum.sumerr() > 0 {
      return Err(Error::New | "");
    }
  }

  cre.set_root(root);

  Ok((cre, sin, bldkind, time))
}

pub fn build_ast_scope(ast_cre: &ast::Krate, sin: &mut StrInterner, far: &Files, imods: &[Imod<'_>]) -> Result<(ScopeMap, Vec<ImplFor>, Duration), Error> {
  let now = Instant::now();
  let ret = ScopeCollector::collect(ast_cre, sin, imods);
  let time = now.elapsed();

  match ret {
    Ok((v1, v2)) => Ok((v1, v2, time)),
    Err(sum) => {
      for emsg in &sum { eprintln!("{}", emsg.display_human(&far)) };
    
      eprint!("{}", sum);

      Err(Error::New | "")
    }
  }
}

pub fn build_hir_krate(ast_cre: &ast::Krate, sin: &StrInterner, far: &Files, ast_scp: &ScopeMap, implst: Vec<ImplFor>, imods: &[hir::CID], ideps: &mut hir::Deps) -> Result<(hir::CID, Duration), Error> {
  let now = Instant::now();
  let (cid, sum) = HGen::low(&ast_cre, sin, far, ast_scp, implst, imods, ideps);
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display_human(&far)) };
    
    eprintln!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "") }
  }
  
  cid.map(|cid| (cid, time)).ok_or(Error::New | "unknown")
}

pub fn build_hir_export(hir_cre: &hir::Krate, deps: &hir::Deps, sin: &StrInterner, far: &Files) -> Result<(ExportMap, Duration), Error> {
  let now = Instant::now();
  let ret = ExportMap::visit(hir_cre, deps, sin);
  let time = now.elapsed();

  match ret {
    Ok(v) => Ok((v, time)),
    Err(sum) => {
      for emsg in &sum { eprintln!("{}", emsg.display_human(&far)) };
    
      eprint!("{}", sum);

      Err(Error::New | "")
    }
  }
}

pub fn build_mir_krate(hir_cre: &hir::Krate, deps: &hir::Deps, sin: &StrInterner, far: &Files, layinfo: &mir::LayoutInfo, expmaps: &[ExportMap]) -> Result<(mir::Krate, Duration), Error> {
  let now = Instant::now();
  let (mir_cre, sum) = MGen::low(&hir_cre, deps, sin, layinfo, expmaps);
  let time = now.elapsed();

  if !sum.is_empty() {
    for emsg in &sum { eprintln!("{}", emsg.display_human(&far)) };
    
    eprint!("{}", sum);

    if sum.sumerr() > 0 { return Err(Error::New | "") }
  }

  Ok((mir_cre.unwrap(), time))
}

pub fn build_cgen(info: &CompileInfo, backend: &Box<dyn ICGen>, mir_cre: &mir::Krate, sin: &StrInterner, ext_ll: bool, outk: OutKind) -> Result<(Vec<u8>, Option<String>, Duration), Error> {
  let now = Instant::now();
  
  let (out, ll) = backend.generate(
    mir_cre,
    sin,
    ext_ll,
    outk,
    &info.triple,
    match info.reloc { RelocMode::PIC => cgen::RelocMode::PIC, RelocMode::Static => cgen::RelocMode::Static },
    match info.mcmodel { CodeModel::Small => cgen::CodeModel::Small, CodeModel::Medium => cgen::CodeModel::Medium, CodeModel::Large => cgen::CodeModel::Large, CodeModel::Kernel => cgen::CodeModel::Kernel },
    match info.opt_level { OptLevel::O0 => Optimization::None, OptLevel::O1 => Optimization::Less, OptLevel::O2 => Optimization::Default, OptLevel::O3 => Optimization::Aggressive },
  ).map_err(|err| Error::Str(err))?;

  let time = now.elapsed();

  Ok((out, ll, time))
}

pub fn build_jit(info: &CompileInfo, backend: &Box<dyn ICGen>, bytecode: &Vec<u8>) -> Result<(), Error> {
  if info.verbose > 0 {
    eprintln!("{}", "Running".green().bold());
  }

  let exitcode = backend.run_vm(&bytecode)?;

  if exitcode != 0 {
    eprintln!("{}{} {}", "Exit Code".green().bold(), ":".bright_black(), exitcode);
  }

  Ok(())
}

pub fn build_link(info: &CompileInfo, outdir: &Path, bld_kind: BuildKind, dep_objs: &[PathBuf]) -> Result<(), Error> {
  if info.verbose > 0 {
    eprintln!("{}", "Linking".green().bold());
  }

  fn get_rtl(rtl: &Option<String>) -> Result<&str, Error> {
    match rtl {
      Some(s) => Ok(s),
      None => {
        let path = Path::new("/usr/lib/qwc/rtl");
        if path.exists() { Ok(path.to_str().unwrap()) } else { Err(Error::New | "rtl sources not found") }
      }
    }
  }
  
  let mut cmd = Command::new("ld.lld");
  
  if bld_kind == BuildKind::Exec {
    let rt = match info.start_routine {
      BuildStartRoutine::CQRT => { cmd.arg("-lc").arg("--dynamic-linker").arg("/lib64/ld-linux-x86-64.so.2"); "cqrt" },
      BuildStartRoutine::CRT  => { cmd.arg("-lc").arg("--dynamic-linker").arg("/lib64/ld-linux-x86-64.so.2"); "crt" },
      BuildStartRoutine::QRT  => "qrt",
    };
  
    cmd.arg(format!("{}/obj/{}.o", get_rtl(&info.rtl)?, rt));
  } else if bld_kind == BuildKind::Lib {
    cmd.arg("-shared");
  }
  
  cmd.arg("-L/usr/lib").arg("-o")
    .arg(match bld_kind {
      BuildKind::Exec => outdir.canonicalize().unwrap().join("out"),
      BuildKind::Lib  => outdir.canonicalize().unwrap().join("out.so"),
    })
    .arg(outdir.canonicalize().unwrap().join("out.o"));

  for dep_obj in dep_objs {
    if dep_obj.exists() {
      cmd.arg(dep_obj);
    }
  }
  
  let out = cmd.output().map_err(|err| err.to_string())?;
  
  if !out.status.success() {
    eprintln!("{}{} {}", "Linker Exit Code".green().bold(), ":".bright_black(), out.status);
    
    io::stderr().write(&out.stdout).map_err(|err| err.to_string())?;
    io::stderr().write(&out.stderr).map_err(|err| err.to_string())?;
  } else {
    io::stderr().write(&out.stdout).map_err(|err| err.to_string())?;
    io::stderr().write(&out.stderr).map_err(|err| err.to_string())?;
  }

  Ok(())
}


pub fn compile(info: CompileInfo) -> Result<(), Error> {
  // Layout
  let hir_layinfo = hir::LayoutInfo{
    str_lay:  hir::Layout::DST,
    bool_lay: hir::Layout::STATIC,
    i8_lay:   hir::Layout::STATIC,
    i16_lay:  hir::Layout::STATIC,
    i32_lay:  hir::Layout::STATIC,
    i64_lay:  hir::Layout::STATIC,
    i128_lay: hir::Layout::STATIC,
    bf16_lay: hir::Layout::STATIC,
    f16_lay:  hir::Layout::STATIC,
    f32_lay:  hir::Layout::STATIC,
    f64_lay:  hir::Layout::STATIC,
    f128_lay: hir::Layout::STATIC,
    ptr_size: hir::Layout::STATIC,
  };

  let mir_layinfo = mir::LayoutInfo{
    arch_bit: mir::ArchBit::B64,
    i8_lay:   mir::Layout::new_sst(1,  1),
    i16_lay:  mir::Layout::new_sst(2,  2),
    i32_lay:  mir::Layout::new_sst(4,  4),
    i64_lay:  mir::Layout::new_sst(8,  8),
    i128_lay: mir::Layout::new_sst(16, 16),
    bf16_lay: mir::Layout::new_sst(1,  1),
    f16_lay:  mir::Layout::new_sst(2,  2),
    f32_lay:  mir::Layout::new_sst(4,  4),
    f64_lay:  mir::Layout::new_sst(8,  8),
    f128_lay: mir::Layout::new_sst(16, 16),
    ptr_size: mir::Layout::new_sst(8, 8),
  };

  let outdir = &info.outdir;
  if !outdir.exists() {
    fs::create_dir_all(outdir)?;
  }

  let mut far = Files::new();

  // PASS 1 (parse)
  if info.verbose > 1 { eprintln!("{}", "PASS 1 (parse)".red().bold()) }
  let (ast_cre, mut sin, bld_kind, time_pass1_parse) = build_ast_krate(info.path, &mut far)?;
  if info.dump.contains(&DumpStage::Ast) { eprintln!("{}", ast::Dump{cre: &ast_cre, sin: &sin, far: &far}) }

  // Core setup
  let mut deps = hir::Deps::new();
  let core_cid = deps.get_next_id();
  let (core_cre, core_exp, prims, prim_op) = qwc_intrinsic::new_core(core_cid, &mut sin, &hir_layinfo);
  deps.add(core_cre);
  deps.set_prims(prims);
  deps.set_prim_op(prim_op);
  let core_name = sin.sid("core");

  // Load external dependency units (.qwu)
  let mut dep_exps = Vec::with_capacity(info.deps.len());
  let mut imod_cids = Vec::with_capacity(1 + info.deps.len());
  imod_cids.push(core_cid);

  let mut dep_objs = Vec::new();

  for dep in &info.deps {
    let mut file = fs::File::open(&dep.qwu_path)
      .map_err(|e| Error::New | format!("cannot open dependency '{}' at {:?}: {}", dep.name, dep.qwu_path, e))?;
    let (dep_hir, dep_exp) = Unit::deserialize(&mut file)
      .map_err(|e| Error::New | format!("cannot deserialize dependency '{}' at {:?}: {}", dep.name, dep.qwu_path, e))?;
    let dep_cid = deps.add(dep_hir);
    imod_cids.push(dep_cid);
    dep_exps.push(dep_exp);

    if let Some(obj) = &dep.obj_path {
      dep_objs.push(obj.clone());
    }
  }

  let mut all_exps = Vec::with_capacity(1 + info.deps.len());
  all_exps.push(core_exp);
  all_exps.extend(dep_exps);

  let mut imods = Vec::with_capacity(all_exps.len());
  imods.push(Imod::new(core_name, &all_exps[0]));
  for (dep, exp) in info.deps.iter().zip(all_exps[1..].iter()) {
    let name_sid = sin.sid(&dep.name);
    imods.push(Imod::new(name_sid, exp));
  }

  // PASS 1 (scope)
  if info.verbose > 1 { eprintln!("{}", "PASS 1 (scope)".red().bold()) }
  let (scp, implst, time_pass1_scope) = build_ast_scope(&ast_cre, &mut sin, &far, &imods)?;
  if info.dump.contains(&DumpStage::Scope) { eprintln!("{}", qwc_resolve::dump_scp::Dump{scp: &scp, sin: &sin, root: ast_cre.root().unwrap().to_any()}) }

  // PASS 2 (hgen)
  if info.verbose > 1 { eprintln!("{}", "PASS 2 (hgen)".red().bold()) }
  let (hir_cid, time_pass2_hgen) = build_hir_krate(&ast_cre, &sin, &far, &scp, implst, &imod_cids, &mut deps)?;
  let hir_cre = deps.get(hir_cid);
  if info.dump.contains(&DumpStage::Hir) { eprintln!("{}", hir::Dump{cre: hir_cre, deps: &deps, sin: &sin}) }

  if info.check_only { return Ok(()) }

  // DROP scp
  drop(scp);

  // PASS 2 (export)
  if info.verbose > 1 { eprintln!("{}", "PASS 2 (export)".red().bold()) }
  let (exp, time_pass2_export) = build_hir_export(hir_cre, &deps, &sin, &far)?;
  if info.dump.contains(&DumpStage::Export) { eprintln!("{}", qwc_resolve::dump_exp::Dump{exp: &exp, cre: hir_cre, deps: Some(&deps), sin: &sin, root: hir_cre.root().unwrap().to_any()}) }

  // QWU save
  let mut qwu = fs::File::create(outdir.join("out.qwu"))?;
  Unit::serialize(&mut qwu, hir_cre, &exp)?;
  qwu.flush()?;

  // PASS 3 (mgen)
  if info.verbose > 1 { eprintln!("{}", "PASS 3 (mgen)".red().bold()) }
  let (mir_cre, time_pass3_mgen) = build_mir_krate(hir_cre, &deps, &sin, &far, &mir_layinfo, &all_exps)?;
  if info.dump.contains(&DumpStage::Mir) { eprint!("{}", mir::Dump{cre: &mir_cre, sin: &sin}) }

  // Choose Backend
  let backend: Box<dyn ICGen> = Box::new(qwc_cgen_llvm::CGen);

  // PASS 4 (cgen)
  if info.verbose > 1 { eprintln!("{}", "PASS 4 (cgen)".red().bold()) }
  let (out, llir, time_pass4_cgen) = build_cgen(
    &info,
    &backend,
    &mir_cre,
    &sin,
    info.dump.contains(&DumpStage::Lir),
    if info.execute { OutKind::ByteCode } else { OutKind::Object },
  )?;

  let bytecode = (info.execute).then_some(&out);
  let object = (!info.execute).then_some(&out);

  if !info.execute {
    fs::write(outdir.join("out.o"), &object.unwrap())?;
  }

  if info.dump.contains(&DumpStage::Lir) {
    let llir = llir.unwrap();
    fs::write(outdir.join("out.ll"), &llir)?;
    eprintln!("{}", llir);
  }

  if info.execute {
    build_jit(&info, &backend, bytecode.unwrap())?;
  } else {
    build_link(&info, outdir, bld_kind, &dep_objs)?;
  }

  // Timings
  if info.timings {
    let pass1 = time_pass1_parse + time_pass1_scope;
    let pass2 = time_pass2_hgen + time_pass2_export;
    let pass3 = time_pass3_mgen;
    let pass4 = time_pass4_cgen;

    eprintln!("{}{} {:?}", "timings".yellow().bold(), ":".bright_black(), (pass1 + pass2 + pass3 + pass4));
    
    if info.verbose > 1 {
      eprintln!("  {}{} {:?}", "pass 1".yellow(), ":".bright_black(), pass1);
      if info.verbose > 2 {
        eprintln!("    {}{} {:?}", "parse".cyan(), ":".bright_black(), time_pass1_parse);
        eprintln!("    {}{} {:?}", "scope".cyan(), ":".bright_black(), time_pass1_scope);
      }
      
      eprintln!("  {}{} {:?}", "pass 2".yellow(), ":".bright_black(), pass2);
      if info.verbose > 2 {
        eprintln!("    {}{} {:?}", "hgen".cyan(), ":".bright_black(), time_pass2_hgen);
        eprintln!("    {}{} {:?}", "export".cyan(), ":".bright_black(), time_pass2_export);
      }
      
      eprintln!("  {}{} {:?}", "pass 3".yellow(), ":".bright_black(), pass3);
      if info.verbose > 2 {
        eprintln!("    {}{} {:?}", "mgen".cyan(), ":".bright_black(), time_pass3_mgen);
      }
      
      eprintln!("  {}{} {:?}", "pass 4".yellow(), ":".bright_black(), pass4);
      if info.verbose > 2 {
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

    if info.verbose > 1 {
      eprintln!("  {}: {} {} {}", "ast".yellow(), humanize_size(ast_used), "/".bright_black(), humanize_size(ast_alloc));
      if info.verbose > 2 {
        eprintln!("    {}: {}", "type".cyan(), humanize_size(ast_cre.size_used::<ast::Type>()));
        eprintln!("    {}: {}", "expr".cyan(), humanize_size(ast_cre.size_used::<ast::Expr>()));
        eprintln!("    {}: {}", "item".cyan(), humanize_size(ast_cre.size_used::<ast::Item>()));
        eprintln!("    {}: {}", "patt".cyan(), humanize_size(ast_cre.size_used::<ast::Patt>()));
        eprintln!("    {}: {}", "field".cyan(), humanize_size(ast_cre.size_used::<ast::Field>()));
        eprintln!("    {}: {}", "thing".cyan(), humanize_size(ast_cre.size_used::<ast::Thing>()));
        eprintln!("    {}: {}", "extra".cyan(), humanize_size(ast_cre.size_used::<ast::AnyId>()));
      }
      
      eprintln!("  {}: {} {} {}", "hir".yellow(), humanize_size(hir_used), "/".bright_black(), humanize_size(hir_alloc));
      if info.verbose > 2 {
        eprintln!("    {}: {}", "type".cyan(), humanize_size(hir_cre.size_used::<hir::Type>()));
        eprintln!("    {}: {}", "expr".cyan(), humanize_size(hir_cre.size_used::<hir::Expr>()));
        eprintln!("    {}: {}", "item".cyan(), humanize_size(hir_cre.size_used::<hir::Item>()));
        eprintln!("    {}: {}", "thing".cyan(), humanize_size(hir_cre.size_used::<hir::Thing>()));
        eprintln!("    {}: {}", "dpath".cyan(), humanize_size(hir_cre.size_used::<hir::DefPath>()));
        eprintln!("    {}: {}", "extra".cyan(), humanize_size(hir_cre.size_used::<hir::AnyId>()));
      }

      eprintln!("  {}: {} {} {}", "mir".yellow(), humanize_size(mir_used), "/".bright_black(), humanize_size(mir_alloc));
      if info.verbose > 2 {
        eprintln!("    {}: {}", "type".cyan(), humanize_size(mir_cre.size_used::<mir::Type>()));
        eprintln!("    {}: {}", "symb".cyan(), humanize_size(mir_cre.size_used::<mir::Symbol>()));
        eprintln!("    {}: {}", "blok".cyan(), humanize_size(mir_cre.size_used::<mir::Block>()));
        eprintln!("    {}: {}", "inst".cyan(), humanize_size(mir_cre.size_used::<mir::Inst>()));
        eprintln!("    {}: {}", "value".cyan(), humanize_size(mir_cre.size_used::<mir::Value>()));
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
