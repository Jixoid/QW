use std::{collections::HashMap, path::Path};

use qwc_arena::Files;
use qwc_ds::Value;

use crate::Error;


pub struct ConfSetup {
  pub name: String,
  pub desc: String,
  pub vers: String,
  
  pub wspace: Option<HashMap<String, String>>,
}

pub fn parse_conf(fpath: &Path, far: &mut Files) -> Result<ConfSetup, Error> {
  let conf = fpath.join("qw.conf");
  if !conf.exists() { return Err(Error::New | "could not find `qw.conf`") }
  
  let fi = {
    let fid = far.add(&conf);
    far.get(fid)
  };
  
  let conf = Value::load_file(fi).map_err(|e| format!("{}", e.display(far)))?;


  struct Setup {
    name: Option<String>,
    desc: Option<String>,
    vers: Option<String>,
    
    wspace: Option<HashMap<String, String>>,
  }

  let mut setup = Setup{name: None, desc: None, vers: None, wspace: None};


  for (name, val) in conf.expect_struct()? {
    match name.as_str() {
      "name" => check2(&mut setup.name, val.expect_string()?)?,
      "desc" => check2(&mut setup.desc, val.expect_string()?)?,
      "vers" => check2(&mut setup.vers, val.expect_string()?)?,

      "deps" => todo!(),
      "feat" => todo!(),

      "wspace" => {
        let raw = val.expect_struct()?;
        let mut map = HashMap::new();

        for (name, val) in raw { map.insert(name.clone(), val.expect_string()?); }

        check2(&mut setup.wspace, map)?
      },

      c @ _ => return Err(Error::New | format!("unknown key: `{}`", c))
    }
  }


  let ret = ConfSetup {
    name: setup.name.unwrap(),
    desc: setup.desc.unwrap(),
    vers: setup.vers.unwrap(),
    wspace: setup.wspace,
  };

  Ok(ret)
}

fn check2<T>(var: &mut Option<T>, val: T) -> Result<(), Error> {
  match var {
    None => { *var = Some(val); Ok(()) },
    Some(..) => { Err(Error::New | "duplicate") },
  }
}


trait EasyAccess {
  fn expect_struct(&self) -> Result<&HashMap<String, Value>, Error>;
  //fn expect_array(&self) -> Result<&Vec<Value>, Error>;
  fn expect_string(&self) -> Result<String, Error>;
}

impl EasyAccess for Value {
  fn expect_struct(&self) -> Result<&HashMap<String, Value>, Error> { if let Value::Stc(ret) = self { Ok(ret) } else { Err(Error::New | "expect_struct") } }
  //fn expect_array(&self) -> Result<&Vec<Value>, Error> { if let Value::Arr(ret) = self { Ok(ret) } else { Err(Error::New | "expect_struct") } }
  fn expect_string(&self) -> Result<String, Error> { if let Value::Str(ret) = self { Ok(ret.clone()) } else { Err(Error::New | "expect_struct") } }
}
