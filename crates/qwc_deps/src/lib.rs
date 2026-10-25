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


fn get_qw_home() -> Option<PathBuf> {
  std::env::var_os("HOME")
    .or_else(|| std::env::var_os("USERPROFILE"))
    .map(|h| PathBuf::from(h).join(".qw"))
}

fn is_valid_git_arg(arg: &str) -> bool {
  !arg.is_empty() && !arg.starts_with('-') && !arg.contains('\0')
}

pub fn find_deps(sum: &mut Summary, path: &Path, deps: &FxHashMap<String, Deps>) -> Status<Vec<(String, PathBuf)>> {
  let wspace = path.join("wspace");
  let wspace = wspace.exists().then_some(wspace.as_path());

  if let Some(wspace) = wspace && !wspace.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wspace.to_str().unwrap()])));
  }

  let Some(wqw) = get_qw_home() else {
    ret_err!(sum, Message::error_nopos(COULD_NOT_DETERMINE_HOME_DIR));
  };

  if wqw.exists() && !wqw.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wqw.to_str().unwrap()])));
  }

  let wgit = wqw.join("git");
  if wgit.exists() && !wgit.is_dir() {
    ret_err!(sum, Message::error_nopos(X_IS_NOT_A_FOLDER.args(&[wgit.to_str().unwrap()])));
  }

  let wremote = wqw.join("remote");
  if wremote.exists() && !wremote.is_dir() {
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

        let dir = wspace.join(name);
        let dir = dir.exists().then_some(dir.as_path());
        let Some(dir) = dir else {
          ret_err!(sum, Message::error_nopos(FOLDER_X_NOT_FOUND.args(&[wspace.join(name).to_str().unwrap()])));
        };

        list.push((entry_name.clone(), dir.to_path_buf()));
      }

      qwc_config::RepoFrom::Git { repo, select } => {
        if !is_valid_git_arg(repo) {
          ret_err!(sum, Message::error_nopos(INVALID_GIT_REPO_URL.args(&[repo])));
        }

        if let Some(select) = select {
          let sel_str = match select {
            qwc_config::GitSelect::Branch(b) => b.as_str(),
            qwc_config::GitSelect::Tag(t) => t.as_str(),
            qwc_config::GitSelect::Rev(r) => r.as_str(),
          };
          if !is_valid_git_arg(sel_str) {
            ret_err!(sum, Message::error_nopos(INVALID_GIT_SPECIFIER.args(&[sel_str])));
          }
        }

        if !wgit.exists() {
          if fs::create_dir_all(&wgit).is_err() {
            ret_err!(sum, Message::error_nopos(FOLDER_X_CAN_NOT_CREATED.args(&[wgit.to_str().unwrap()])));
          }
        }

        let hash = &mut FxHasher::default();
        repo.hash(hash);
        select.hash(hash);
        let hash = hash.finish();

        let dir = wgit.join(format!("{:x}", hash));
        if dir.exists() {
          list.push((entry_name.clone(), dir));
          continue;
        }

        // Staging directory for atomic cloning
        let tmp_dir = wgit.join(format!(".tmp_{:x}_{}", hash, std::process::id()));
        if tmp_dir.exists() {
          let _ = fs::remove_dir_all(&tmp_dir);
        }
        if fs::create_dir_all(&tmp_dir).is_err() {
          ret_err!(sum, Message::error_nopos(FOLDER_X_CAN_NOT_CREATED.args(&[tmp_dir.to_str().unwrap()])));
        }

        let cleanup = |tmp: &Path| {
          let _ = fs::remove_dir_all(tmp);
        };

        // Clone Repo with `--` separator
        let cmd = match Command::new("git")
          .current_dir(&tmp_dir)
          .args(["clone", "-q", "--"])
          .arg(repo)
          .arg(".")
          .output()
        {
          Ok(v) => v,
          Err(..) => {
            cleanup(&tmp_dir);
            ret_err!(sum, Message::error_nopos(TOOL_X_NOT_FOUND_IN_SYSTEM.args(&["git"])));
          }
        };

        if !cmd.status.success() {
          cleanup(&tmp_dir);
          let err = String::from_utf8_lossy(&cmd.stderr).trim().to_string();
          ret_err!(sum, Message::error_nopos(GIT_REPO_CAN_NOT_CLONED) + (ERROR_X.args(&[&err])));
        }

        // Select Specific branch / tag / rev with `--` separator
        if let Some(select) = select {
          let mut cmd = Command::new("git");
          let mut cmd = cmd.current_dir(&tmp_dir);

          match select {
            qwc_config::GitSelect::Branch(branch) => {
              cmd = cmd.args(["switch", "--"]).arg(branch);
            }
            qwc_config::GitSelect::Tag(tag) => {
              cmd = cmd.args(["checkout", "--"]).arg(tag);
            }
            qwc_config::GitSelect::Rev(rev) => {
              cmd = cmd.args(["checkout", "--"]).arg(rev);
            }
          }

          let out = match cmd.output() {
            Ok(v) => v,
            Err(..) => {
              cleanup(&tmp_dir);
              ret_err!(sum, Message::error_nopos(TOOL_X_NOT_FOUND_IN_SYSTEM.args(&["git"])));
            }
          };

          if !out.status.success() {
            cleanup(&tmp_dir);
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            ret_err!(sum, Message::error_nopos(GIT_REPO_CAN_NOT_CLONED) + (ERROR_X.args(&[&err])));
          }
        }

        // Atomically move tmp_dir to target dir
        if fs::rename(&tmp_dir, &dir).is_err() {
          cleanup(&tmp_dir);
          ret_err!(sum, Message::error_nopos(FOLDER_X_CAN_NOT_CREATED.args(&[dir.to_str().unwrap()])));
        }

        list.push((entry_name.clone(), dir));
      }

      qwc_config::RepoFrom::Remote { .. } => {
        ret_err!(sum, Message::error_nopos(REMOTE_DEPS_NOT_SUPPORTED));
      }
    }
  }

  Success(list)
}
