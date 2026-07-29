use std::{env, fs, path::PathBuf};
use uuid::Uuid;

use crate::{control::module::{self}, ds::Value};



pub struct InitInfo {
  pub path: String,
  pub name: String,
  pub desc: String,
  pub no_git: bool,
}


pub fn init(info: InitInfo) -> module::Result<()> {

  let path = if info.path.is_empty() {
    env::current_dir()?
  } else {
    PathBuf::from(&info.path)
  };

  if !path.exists() { return Err(module::CompilerError::Str(format!("the path does not exist: {}", path.display()))); }

  if !path.is_dir() { return Err(module::CompilerError::Str(format!("the path is not a directory: {}", path.display()))); }

  let is_not_empty = fs::read_dir(&path)?.next().is_some();

  if is_not_empty { return Err(module::CompilerError::Str("the directory is not empty".to_string())); }
  
  //: Path Configured

  let mut conf = Value::make_struct();

  let default_name = match path.canonicalize() {
    Ok(p) => p.file_name().and_then(|n| n.to_str()).unwrap_or("project name").to_string(),
    Err(_) => path.file_name().and_then(|n| n.to_str()).unwrap_or("project name").to_string(),
  };
  let proj_name = if info.name.is_empty() { &default_name } else { &info.name };

  conf.push_struct("uuid", Value::make_string(Uuid::new_v4().to_string()) )?;
  conf.push_struct("name", Value::make_str(proj_name))?;
  conf.push_struct("desc", Value::make_str(if info.desc.is_empty() { "" } else { &info.desc }))?;
  conf.push_struct("vers", Value::make_str("0.1.0"))?;
  
  conf.push_struct("onerepo", Value::make_struct())?;

  conf.push_struct("deps", Value::make_struct())?;

  conf.save_file(String::from(match path.join("qw.conf").to_str() {
    Some(r) => r,
    None => return Err(module::CompilerError::Str("cannot write: qw.conf".to_string())),
  }))?;

  //: Manifest Writed
  
  let src_dir = path.join("src");
  fs::create_dir_all(&src_dir)?;

  let main_content = "\nfun main() -> i32 {\n  ret 0;\n}\n";
  fs::write(src_dir.join("main.qw"), main_content)?;

  //: Writed Basic Files

  if !info.no_git {
    fs::write(path.join(".gitignore"), "/build\n")?;
    
    let status = std::process::Command::new("git")
      .arg("-C")
      .arg(&path)
      .args(["init", "-q"])
      .status()
      .map_err(|e| module::CompilerError::Str(format!("Git çalıştırılamadı: {e}")))?;

    if !status.success() {
      return Err(module::CompilerError::Str("git repo cannot be initialized".to_string()));
    }
  };

  //: Git Initialized

  Ok(())
}
