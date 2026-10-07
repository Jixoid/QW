use std::path::Path;

use crate::{BuildStartRoutine, BuildVariant, DumpStage, Error, build};


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
  })
}
