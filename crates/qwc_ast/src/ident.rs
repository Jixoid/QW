use std::num::NonZeroU16;

use qwc_arena::Files;
use qwc_diagnostic::Span;
use qwc_string_interner::Sid;


#[derive(Debug, Clone, Copy)]
pub struct Ident {
  off: u32,
  len: NonZeroU16,
  fid: u16,
  sid: Sid,
}

impl Ident {

  pub fn new(off: u32, len: NonZeroU16, fid: u16, sid: Sid) -> Self {
    Self{off, len, fid, sid}
  }

  
  pub fn str<'a>(&self, far: &'a Files) -> &'a str {
    let rng = (self.off as usize)..((self.off as usize)+(self.len.get() as usize));

    let a = &far.get(self.fid).map()[rng];

    str::from_utf8(a).unwrap()
  }
  
  pub fn fid(&self) -> u16 { self.fid }
  pub fn sid(&self) -> Sid { self.sid }
}

impl Into<Span> for Ident {

  fn into(self) -> Span {
    Span::new(self.off, self.len, self.fid)
  }

}
