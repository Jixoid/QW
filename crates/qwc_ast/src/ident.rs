use std::num::{NonZeroU16, NonZeroU32};
use qwc_diagnostic::Span;


#[derive(Clone, Copy)]
pub struct Ident {
  off: u32,
  len: NonZeroU16,
  fid: u16,
  sid: NonZeroU32,
}

impl Ident {

  pub fn new(off: u32, len: NonZeroU16, fid: u16, sid: NonZeroU32) -> Self {
    Self{off, len, fid, sid}
  }

  pub fn fid(&self) -> u16 { self.fid }
  pub fn sid(&self) -> NonZeroU32 { self.sid }

}

impl Into<Span> for Ident {

  fn into(self) -> Span {
    Span::new(self.off, self.len, self.fid)
  }

}
