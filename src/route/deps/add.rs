use std::path::PathBuf;

use crate::{control::ModuleFile, ds::Value};


pub struct AddInfo {
  pub path: String,
  pub name: String,
  pub url: String,
}


pub fn add(info: AddInfo) -> Result<(), String> {
  let path = info.path;
  let name = info.name;
  let url = info.url;
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
  
  let mut conf = Value::load_file(&mfd)?;

  if let Value::Stc(ref mut stc) = conf {
    let mut found = false;
    for field in &mut stc.subs {
      if field.name == "onerepo" {
        if let Value::Stc(ref mut repos) = field.kind {
          // Check if it already exists
          let mut exists = false;
          for repo in &mut repos.subs {
            if repo.name == name {
              repo.kind = Value::Str(url.clone());
              exists = true;
              break;
            }
          }
          if !exists {
            repos.subs.push(crate::ds::SubTypeStc {
              name: name.clone(),
              kind: Value::Str(url.clone()),
            });
          }
        }
        found = true;
        break;
      }
    }
    if !found {
      return Err("could not find `onerepo` section in `qw.conf`.".to_string());
    }
  }

  conf.save_file(conf_path.to_str().unwrap_or("").to_string())?;
  println!("added dependency '{}' to `qw.conf`.", name);

  Ok(())
}
