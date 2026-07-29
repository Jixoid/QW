use std::path::PathBuf;

use crate::{control::ModuleFile, ds::Value};


pub struct SyncInfo {
  pub path: String,
}


pub fn sync(info: SyncInfo) -> Result<(), String> {
  let path = info.path;
  let proj_path = if path.is_empty() {
    std::env::current_dir().map_err(|e| e.to_string())?
  } else {
    PathBuf::from(path)
  };

  let conf_path = proj_path.join("qw.conf");
  if !conf_path.exists() {
    return Err("could not find `qw.conf`.".to_string());
  }

  let mmap = std::fs::read_to_string(&conf_path).map_err(|e| e.to_string())?;
  let mfd = ModuleFile {
    fpath: conf_path.to_str().unwrap_or("").to_string(),
    mmap,
    kind: crate::control::module::ModuleKind::Regular,
  };
  
  let conf = Value::load_file(&mfd)?;

  if let Value::Stc(stc) = conf {
    let mut onerepo = None;
    for field in &stc.subs {
      if field.name == "onerepo" {
        onerepo = Some(&field.kind);
        break;
      }
    }

    if let Some(Value::Stc(repos)) = onerepo {
      let deps_dir = proj_path.join("build").join("deps");
      std::fs::create_dir_all(&deps_dir).map_err(|e| format!("failed to create deps directory: {}", e))?;

      for repo in &repos.subs {
        let repo_name = &repo.name;
        if let Value::Str(url) = &repo.kind {
          let target_dir = deps_dir.join(repo_name);
          
          println!("syncing {} from {}...", repo_name, url);
          
          if target_dir.exists() {
            let status = std::process::Command::new("git")
              .arg("-C")
              .arg(&target_dir)
              .arg("pull")
              .status()
              .map_err(|e| format!("failed to run git pull for {}: {}", repo_name, e))?;
              
            if !status.success() {
              return Err(format!("failed to update dependency {}", repo_name));
            }
          } else {
            let status = std::process::Command::new("git")
              .arg("clone")
              .arg(url)
              .arg(&target_dir)
              .status()
              .map_err(|e| format!("failed to run git clone for {}: {}", repo_name, e))?;
              
            if !status.success() {
              return Err(format!("failed to clone dependency {}", repo_name));
            }
          }
        }
      }
    }
  }

  println!("dependencies synced successfully.");
  Ok(())
}
