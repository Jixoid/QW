/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{fs, path::{Path, PathBuf}};
use qwc_arena::Files;
use qwc_diagnostic::Summary;

use crate::Error;


#[derive(Clone)]
pub struct CleanInfo<'a> {
  pub path: &'a Path,
}

pub fn clean(info: CleanInfo) -> Result<(), Error> {
  let outdir: PathBuf = info.path.canonicalize().unwrap_or_else(|_| info.path.to_path_buf()).join("build");

  clear_directory(outdir)?;

  let mut far = Files::default();

  // Config
  let conf = qwc_config::read_conf(&info.path.canonicalize().unwrap_or_else(|_| info.path.to_path_buf()), &mut far)?;

  if !conf.deps.is_empty() {
    let mut sum = Summary::new();
    let deps = qwc_deps::find_deps(&mut sum, info.path, &conf.deps);

    for msg in sum {
      eprintln!("{}", msg.display_human(&far));
    }

    let deps = deps.ok_or_else(|| Error::New | "dependencies not resolved")?;

    for (_, path) in &deps {
      let mut sub_info = info.clone();
      sub_info.path = path;

      clean(sub_info)?;
    }
  }

  Ok(())
}


pub fn clear_directory<P: AsRef<Path>>(dir_path: P) -> std::io::Result<()> {
  for entry in fs::read_dir(dir_path)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_dir() {
      fs::remove_dir_all(&path)?;
    } else {
      fs::remove_file(&path)?;
    }
  }
  Ok(())
}
