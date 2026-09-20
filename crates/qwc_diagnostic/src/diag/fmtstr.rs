
pub struct FmtMsg {
  pub(crate) msg: &'static str,
}

pub struct FmtString {
  pub(crate) msg: String,
}


impl FmtMsg {
  pub const fn new(msg: &'static str) -> Self {
    Self{msg}
  }

  pub fn args(self, args: &[&str]) -> FmtString {
    FmtString { msg: format_from_vector(self.msg, args) }
  }
}

impl Into<FmtString> for FmtMsg {
  fn into(self) -> FmtString {
    FmtString { msg: self.msg.to_string() }
  }
}


fn format_from_vector(str: &str, args: &[&str]) -> String {
  let mut res = String::new();
  let mut parts = str.split("{}");
  
  if let Some(first) = parts.next() {
    res.push_str(first);
  }
  
  let mut v_iter = args.iter();
  for part in parts {
    if let Some(arg) = v_iter.next() {
      res.push_str(arg);
    }
    res.push_str(part);
  }
  
  res
}
