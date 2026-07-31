use std::env;
use std::fs;
use std::path::PathBuf;


pub struct InstallInfo {
  pub path: String,
  pub force: bool,
}


pub fn install(info: InstallInfo) -> Result<(), String> {
  let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
  let qw_bin_dir = PathBuf::from(home_dir).join(".qw").join("bin");

  if !qw_bin_dir.exists() {
    fs::create_dir_all(&qw_bin_dir).map_err(|e| format!("Failed to create ~/.qw/bin: {}", e))?;
  }

  //: Access ~/.qw/bin

  let conf_path = std::path::Path::new(&info.path).join("qw.conf");
  if !conf_path.exists() {
    return Err("could not find `qw.conf`.".to_string());
  }

  let mmap = fs::read_to_string(&conf_path).map_err(|e| e.to_string())?;
  let mfd = crate::control::module::ModuleFile {
    fpath: conf_path.to_str().unwrap_or("").to_string(),
    mmap,
    kind: crate::control::module::ModuleKind::Regular,
  };
  
  let conf = crate::ds::Value::load_file(&mfd).map_err(|e| format!("{:?}", e))?;

  let project_name = || -> Result<String, String> {
    if let crate::ds::Value::Stc(stc) = conf {
      for field in &stc.subs {
        if field.name == "name" {
          if let crate::ds::Value::Str(s) = &field.kind {
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
    return Err(format!("binary '{}' already exists in ~/.qw/bin", project_name));
  }

  let build_info = crate::route::build::BuildInfo{
    path: &info.path,
    variant: crate::BuildVariant::Release,
    verbose: false,
    timings: false,
    ast_dump: false,
    hir_dump: false,
    check_only: false,
  };

  crate::route::build::build(build_info).map_err(|e| format!("Build failed: {:?}", e))?;


  Ok(())
}
