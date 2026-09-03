use std::{env, fs, path::PathBuf};

use qwc_arena::Files;
use qwc_ds::Value;

use crate::Error;


pub struct InstallInfo {
  pub path: String,
  pub force: bool,
}

pub fn install(info: InstallInfo) -> Result<(), Error> {
  let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set")?;
  let qw_bin_dir = PathBuf::from(home_dir).join(".qw").join("bin");

  if !qw_bin_dir.exists() {
    fs::create_dir_all(&qw_bin_dir).map_err(|e| format!("Failed to create ~/.qw/bin: {}", e))?;
  }

  //: Access ~/.qw/bin

  let mut far = Files::new();

  //: Files

  let conf_path = std::path::Path::new(&info.path).join("qw.conf");
  if !conf_path.exists() {
    return Err(Error::New | "could not find `qw.conf`.");
  }

  let mfd = far.add(&conf_path);
  
  let conf = Value::load_file(&mfd).map_err(|e| format!("{}", e.display(&far)))?;

  let project_name = || -> Result<String, String> {
    if let Value::Stc(stc) = conf {
      for (name, val) in &stc {
        if name == "name" {
          if let Value::Str(s) = &val {
            return Ok(s.clone());
          }
        }
      }
    }
    
    Err("name field not found in qw.conf".to_string())
  }()?;

  //: Read qw.conf

  let target_bin = qw_bin_dir.join(&project_name);

  if !info.force && target_bin.exists() {
    return Err(Error::New | format!("binary '{}' already exists in ~/.qw/bin", project_name));
  }

  let build_info = crate::build::BuildInfo{
    path: &info.path,
    variant: crate::BuildVariant::Release,
    verbose: false,
    timings: false,
    usages: false,
    check_only: false,
  };

  crate::build::build(build_info).map_err(|e| format!("Build failed: {:?}", e))?;

  Ok(())
}
