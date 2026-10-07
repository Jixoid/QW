/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#[derive(Debug, Clone)]
pub struct CodedMsg {
  pub(crate) code: Option<u16>,
  pub(crate) msg: &'static str,
}

#[derive(Debug, Clone)]
pub struct CodedString {
  pub(crate) code: Option<u16>,
  pub(crate) msg: String,
}


impl CodedMsg {
  pub const fn new(code: u16, msg: &'static str) -> Self {
    Self{code: Some(code), msg}
  }

  pub const fn new_str(msg: &'static str) -> Self {
    Self{code: None, msg}
  }


  pub fn args(self, args: &[&str]) -> CodedString {
    CodedString { code: self.code, msg: format_from_vector(self.msg, args) }
  }
}

impl Into<CodedString> for CodedMsg {
  fn into(self) -> CodedString {
    CodedString { code: self.code, msg: self.msg.to_string() }
  }
}

impl Into<String> for CodedMsg {
  fn into(self) -> String {
    if let Some(code) = self.code {
      format!("[{}]{}", code, self.msg)
    } else {
      self.msg.to_string()
    }
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
