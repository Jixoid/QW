use qwc_string_interner::{Sid, StrInterner};


pub trait Mangler {
  fn new(sin: &StrInterner, mgr: &[Sid], now: Sid) -> String;
}


pub struct ManglerQW;

impl Mangler for ManglerQW {
  fn new(sin: &StrInterner, mgr: &[Sid], now: Sid) -> String {
    let mut ret = String::from("qw_");

    for sid in mgr {
      let str = sin.str(*sid);
      ret += &format!("{}{}", str.len(), str);
    }

    let str = sin.str(now);
    ret += &format!("{}{}", str.len(), str);

    ret
  }
}
