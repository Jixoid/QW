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

  if !path.exists() { panic!("the path does not exist: {}", path.display()); }

  if !path.is_dir() { panic!("the path is not a directory: {}", path.display()); }

  let is_not_empty = fs::read_dir(&path)?.next().is_some();

  if is_not_empty { panic!("the directory is not empty"); }
  
  //: Path Configured

  let mut conf = Value::make_struct();

  conf.push_struct("uuid", Value::make_string(Uuid::new_v4().to_string()) )?;
  conf.push_struct("name", Value::make_str(if info.name.is_empty() { "project name" } else { &info.name }))?;
  conf.push_struct("desc", Value::make_str(if info.desc.is_empty() { "project description" } else { &info.desc }))?;
  conf.push_struct("vers", Value::make_str("0.1.0"))?;
  
  let mut onerepo = Value::make_struct();
  onerepo.push_struct("qw", Value::make_str("https://github.com/jixoid/qw"))?;
  conf.push_struct("onerepo", onerepo)?;

  let mut deps = Value::make_struct();
  deps.push_struct("qw", Value::make_str("0.1.0"))?;
  conf.push_struct("deps", deps)?;

  conf.save_file(String::from(match path.join("qw.conf").to_str() {
    Some(r) => r,
    None => panic!("cannot writed: qw.conf"),
  }))?;

  //: Manifest Writed
  
  let src_dir = path.join("src");
  fs::create_dir_all(&src_dir)?;

  let main_content = "\nfun main() -> i32 {\n  ret 0;\n}\n";
  fs::write(src_dir.join("main.qw"), main_content)?;

  //: Writed Basic Files

  if !info.no_git {
    fs::write(path.join(".gitignore"), "/build\n/.deps\n")?;
    
    let status = std::process::Command::new("git")
      .arg("-C")
      .arg(path)
      .args(["init", "-q"])
      .status()
      .map_err(|e| panic!("Git çalıştırılamadı: {e}"));

    match status {
      Ok(_r) => (),
      Err(_e) => panic!("git repo cannot Initialized"),
    };
  };

  //: Git Initialized

  Ok(())
}
