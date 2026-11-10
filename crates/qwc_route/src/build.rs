/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use std::path::{Path, PathBuf};
use qwc_arena::Files;
use qwc_diagnostic::Summary;
use qwc_target::{Abi, Sys, Triple, Vendor, get_dialect};

use crate::{compile, BuildStartRoutine, BuildVariant, CodeModel, DumpStage, Error, OptLevel, RelocMode};


#[derive(Clone)]
pub struct BuildInfo<'a> {
  pub path: &'a Path,
  pub opt_level: OptLevel,
  pub variant: BuildVariant,
  pub start_routine: BuildStartRoutine,
  pub rtl: Option<String>,
  pub reloc: RelocMode,
  pub mcmodel: CodeModel,
  pub verbose: u8,
  pub timings: bool,
  pub usages: bool,
  pub dump: Vec<DumpStage>,
  pub check_only: bool,
  pub execute: bool,
  pub dolink: bool,
}


pub fn build(info: BuildInfo) -> Result<(), Error> {
  let variant_dir = match info.variant {
    BuildVariant::Debug => "debug",
    BuildVariant::Release => "release",
    BuildVariant::RelWithDebInfo => "relwithdebinfo",
  };

  let outdir: PathBuf = info.path.canonicalize().unwrap_or_else(|_| info.path.to_path_buf())
    .join("build")
    .join(variant_dir);

  let mut far = Files::default();

  // Config
  let conf = qwc_config::read_conf(&info.path.canonicalize().unwrap_or_else(|_| info.path.to_path_buf()), &mut far)?;

  let triple = Triple(conf.arch, Vendor::Unknown, Sys::Linux, Abi::Unknown);

  let dialect = get_dialect(triple);


  let mut dep_artifacts = Vec::new();

  if !conf.deps.is_empty() {
    let mut sum = Summary::new();
    let deps = qwc_deps::find_deps(&mut sum, info.path, &conf.deps);

    for msg in sum {
      eprintln!("{}", msg.display_human(&far));
    }

    let deps = deps.ok_or_else(|| Error::New | "dependencies not resolved")?;

    for (name, path) in &deps {
      let mut sub_info = info.clone();
      sub_info.path = path;
      sub_info.dump = vec![];
      sub_info.execute = false;
      sub_info.dolink = false;

      build(sub_info)?;

      let dep_outdir = path.canonicalize().unwrap_or_else(|_| path.clone())
        .join("build")
        .join(variant_dir);

      dep_artifacts.push(compile::DepArtifact {
        name: name.clone(),
        path: dep_outdir,
      });
    }
  }

  let compile_info = compile::CompileInfo {
    path: info.path,
    outdir,
    dialect,
    triple,
    opt_level: info.opt_level,
    variant: info.variant,
    start_routine: info.start_routine,
    rtl: info.rtl,
    reloc: info.reloc,
    mcmodel: info.mcmodel,
    link: crate::LinkMode::Exec,
    verbose: info.verbose,
    timings: info.timings,
    usages: info.usages,
    dump: info.dump,
    check_only: info.check_only,
    execute: info.execute,
    deps: dep_artifacts,
    krate_name: Some(conf.name.clone()),
    dolink: info.dolink,
  };

  compile::compile(compile_info)
}
