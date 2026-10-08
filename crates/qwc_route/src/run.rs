/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;

use crate::{BuildStartRoutine, BuildVariant, CodeModel, DumpStage, Error, OptLevel, RelocMode, build};


pub struct RunInfo<'a> {
  pub path: &'a Path,
  pub verbose: u8,
  pub timings: bool,
  pub usages: bool,
  pub dump: Vec<DumpStage>,
}

pub fn run(info: RunInfo) -> Result<(), Error> {
  build::build(build::BuildInfo {
    path: info.path,
    opt_level: OptLevel::O0,
    verbose: info.verbose,
    timings: info.timings,
    usages: info.usages,
    dump: info.dump,
    check_only: false,
    execute: true,
    variant: BuildVariant::Debug,
    start_routine: BuildStartRoutine::CRT,
    triple: None,
    rtl: None,
    reloc: RelocMode::PIC,
    mcmodel: CodeModel::Small,
  })
}
