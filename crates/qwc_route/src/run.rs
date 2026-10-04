use std::path::Path;

use crate::{BuildVariant, DumpStage, Error, build};


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
  })
}
