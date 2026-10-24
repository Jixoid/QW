/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


mod version;
mod config;

use std::{collections::HashMap, path::Path, str::FromStr};

use qwc_arena::Files;
use qwc_diagnostic::{Summary, Status::{Fail, Success}};
use qwc_ds::Value;
use rustc_hash::{FxHashMap, FxHashSet};

pub use version::*;
pub use config::*;


pub fn read_conf(fpath: &Path, far: &mut Files) -> Result<Config, String> {
  let conf = fpath.join("qw.conf");
  if !conf.exists() { return Err("could not find `qw.conf`".to_string()) }
  
  let fi = {
    let fid = far.add(&conf);
    far.get(fid)
  };
  
  let conf = {
    let mut sum = Summary::new();

    match Value::load_file(fi, &mut sum) {
      Fail => return Err(format!("{}", sum)),
      Success(v) => v,
    }
  };

  struct Setup {
    name: Option<String>,
    vers: Option<Version>,
    
    feat: Option<FxHashSet<String>>,
    deps: Option<FxHashMap<String, Deps>>,
  }

  let mut setup = Setup{name: None, vers: None, feat: None, deps: None};

  let dep_read = |val: &Value| -> Result<Deps, String> {
    let mut feat = None;
    let mut from = None;

    for (name, val) in val.expect_struct()? {
      match name.as_str() {
        "wspace" => check2(&mut from, RepoFrom::WSpace { name: val.expect_string()? })?,

        "git" => {
          let mut repo = None;
          let mut select = None;
          
          for (name, val) in val.expect_struct()? {
            match name.as_str() {
              "repo" => check2(&mut repo, val.expect_string()?)?,

              "branch" => check2(&mut select, GitSelect::Branch(val.expect_string()?))?,
              "tag" => check2(&mut select, GitSelect::Tag(val.expect_string()?))?,
              "rev" => check2(&mut select, GitSelect::Rev(val.expect_string()?))?,

              c @ _ => return Err(format!("unknown key: `{}`", c))
            }
          }

          let repo = repo.ok_or("`repo` not specified")?;
          
          check2(&mut from, RepoFrom::Git { repo, select })?
        }

        "remote" => {
          let mut verreq = None;
          let mut remote = None;
          
          for (name, val) in val.expect_struct()? {
            match name.as_str() {
              "ver" => check2(&mut verreq, VerReq::from_str(&val.expect_string()?).map_err(|err| format!("{err}"))?)?,

              "remote" => check2(&mut remote, val.expect_string()?)?,
              
              c @ _ => return Err(format!("unknown key: `{}`", c))
            }
          }
          
          check2(&mut from, RepoFrom::Remote { verreq, remote })?
        }

        "feat" => {
          let mut set = FxHashSet::default();

          for name in val.expect_array()? { set.insert(name.expect_string()?); }

          check2(&mut feat, set)?
        }

        c @ _ => return Err(format!("unknown key: `{}`", c))
      }
    }

    Ok(Deps{feat, from: from.ok_or_else(|| "`from` not specified")?})
  };

  for (name, val) in conf.expect_struct()? {
    match name.as_str() {
      "name" => check2(&mut setup.name, val.expect_string()?)?,
      "desc" => {}, //check2(&mut setup.desc, val.expect_string()?)?,
      "vers" => check2(&mut setup.vers, Version::from_str(&val.expect_string()?).map_err(|err| format!("{err}"))?)?,

      "feat" => {
        let mut set = FxHashSet::default();

        for name in val.expect_array()? {
          set.insert(name.expect_string()?);
        }

        check2(&mut setup.feat, set)?
      }

      "deps" => {
        let mut map = FxHashMap::default();

        for (name, val) in val.expect_struct()? {
          let deps = dep_read(val)?;
          
          map.insert(name.clone(), deps);
        }
        
        check2(&mut setup.deps, map)?
      }

      c @ _ => return Err(format!("unknown key: `{}`", c))
    }
  }


  let ret = Config {
    name: setup.name.unwrap(),
    //desc: setup.desc.unwrap(),
    vers: setup.vers.unwrap(),
    feat: setup.feat.unwrap_or_else(|| FxHashSet::default()),
    deps: setup.deps.unwrap_or_else(|| FxHashMap::default()),
  };

  Ok(ret)
}

fn check2<T>(var: &mut Option<T>, val: T) -> Result<(), &str> {
  match var {
    None => { *var = Some(val); Ok(()) },
    Some(..) => { Err("duplicate") },
  }
}


trait EasyAccess {
  fn expect_struct(&self) -> Result<&HashMap<String, Value>, &str>;
  fn expect_array(&self) -> Result<&Vec<Value>, &str>;
  fn expect_string(&self) -> Result<String, &str>;
}

impl EasyAccess for Value {
  fn expect_struct(&self) -> Result<&HashMap<String, Value>, &str> { if let Value::Stc(ret) = self { Ok(ret) } else { Err("expect_struct") } }
  fn expect_array(&self) -> Result<&Vec<Value>, &str> { if let Value::Arr(ret) = self { Ok(ret) } else { Err("expect_struct") } }
  fn expect_string(&self) -> Result<String, &str> { if let Value::Str(ret) = self { Ok(ret.clone()) } else { Err("expect_struct") } }
}
