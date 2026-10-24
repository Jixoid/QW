/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{fs, hash::{Hash, Hasher}, path::{Path, PathBuf}, process::Command};

use qwc_config::Deps;
use qwc_diagnostic::{Message, Status::{self, Success}, Summary, msg::*, ret_err};
use rustc_hash::{FxHashMap, FxHasher};


pub fn find_deps(sum: &mut Summary, path: &Path, deps: &FxHashMap<String, Deps>) -> Status<Vec<(String, PathBuf)>> {
  let wspace = path.join("wspace");
  let wspace = wspace.exists().then_some(wspace.as_path());

  if let Some(wspace) = wspace && !wspace.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wspace.to_str().unwrap()])));
  }


  let wqw = PathBuf::from(env!("HOME")).join(".qw");
  let wqw = wqw.exists().then_some(wqw);

  if let Some(wqw) = wqw && !wqw.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wqw.to_str().unwrap()])));
  }


  let wgit = PathBuf::from(env!("HOME")).join(".qw").join("git");
  let wgit = wgit.exists().then_some(wgit.as_path());

  if let Some(wgit) = wgit && !wgit.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wgit.to_str().unwrap()])));
  }


  let wremote = PathBuf::from(env!("HOME")).join(".qw").join("remote");
  let wremote = wremote.exists().then_some(wremote);

  if let Some(wremote) = wremote && !wremote.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wremote.to_str().unwrap()])));
  }


  
  // Find Deps
  let mut list: Vec<(String, PathBuf)> = vec![];


  for (entry_name, dep) in deps {
    match &dep.from {
      qwc_config::RepoFrom::WSpace { name } => {
        let Some(wspace) = wspace else {
          ret_err!(sum, Message::error_nopos(FOLDER_X_NOT_FOUND.args(&[path.join("wspace").to_str().unwrap()])));
        };

        let dir = wspace.join(&name);
        let dir = dir.exists().then_some(dir.as_path());
        let Some(dir) = dir else {
          ret_err!(sum, Message::error_nopos(FOLDER_X_NOT_FOUND.args(&[wspace.join(&name).to_str().unwrap()])));
        };

        list.push((entry_name.clone(), dir.to_path_buf()));
      }

      qwc_config::RepoFrom::Git { repo, select } => {
        let Some(wgit) = wgit else {
          ret_err!(sum, Message::error_nopos(FOLDER_X_NOT_FOUND.args(&[PathBuf::from(env!("HOME")).join(".qw").join("git").to_str().unwrap()])));
        };

        let hash = &mut FxHasher::default();
        repo.hash(hash);
        select.hash(hash);
        let hash = hash.finish();

        let dir = wgit.join(format!("{:x}", hash));
        if dir.exists() {
          list.push((entry_name.clone(), dir.to_path_buf()));
          continue;
        } else {
          if let Err(..) = fs::create_dir(&dir) {
            ret_err!(sum, Message::error_nopos(FOLDER_X_CAN_NOT_CREATED.args(&[dir.to_str().unwrap()])));
          }
        }
        
        
        // Clone Repo
        let cmd = match Command::new("git").current_dir(&dir).arg("clone").arg("-q").arg(repo).arg(".").output() {
          Ok(v) => v,
          Err(..) => ret_err!(sum, Message::error_nopos(TOOL_X_NOT_FOUND_IN_SYSTEM.args(&["git"]))),
        };

        if !cmd.status.success() {
          let err = String::from_utf8(cmd.stderr).unwrap();
          ret_err!(sum, Message::error_nopos(GIT_REPO_CAN_NOT_CLONED).add(ERROR_X.args(&[&format!("{err}")])));
        }


        // Select Specific
        if let Some(select) = select {
          let mut cmd = Command::new("git");
          let mut cmd = cmd.current_dir(&dir);

          match select {
            qwc_config::GitSelect::Branch(branch) => cmd = cmd.arg("switch").arg(branch),

            qwc_config::GitSelect::Tag(tag) => cmd = cmd.arg("checkout").arg(tag),
            qwc_config::GitSelect::Rev(rev) => cmd = cmd.arg("checkout").arg(rev),
          }

          let out = match cmd.output() {
            Ok(v) => v,
            Err(..) => ret_err!(sum, Message::error_nopos(TOOL_X_NOT_FOUND_IN_SYSTEM.args(&["git"]))),
          };

          if !out.status.success() {
            let err = String::from_utf8(out.stderr).unwrap();
            ret_err!(sum, Message::error_nopos(GIT_REPO_CAN_NOT_CLONED).add(ERROR_X.args(&[&format!("{err}")])));
          }
        }

        list.push((entry_name.clone(), dir.to_path_buf()));
      }
      
      _ => todo!()
    }
  }

  Success(list)
}
