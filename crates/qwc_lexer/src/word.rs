use std::num::NonZeroU16;
use qwc_diagnostic::Span;

use crate::wkind::WK;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Word {
  pub(crate) off: u32,
  pub(crate) len: NonZeroU16,
  pub(crate) fid: u16,
  pub(crate) kind: WK,
}

impl Word {

  pub fn new(off: u32, len: NonZeroU16, fid: u16, kind: WK) -> Self {
    Self{off, len, fid, kind}
  }
  
  pub(crate) fn new_safe(off: usize, len: usize, fid: u16, kind: WK) -> Self {
    Self{off: u32::try_from(off).unwrap(), len: NonZeroU16::new(u16::try_from(len).unwrap()).unwrap(), fid, kind}
  }


  #[cfg(feature = "ast-internal")]
  pub fn to(&self) -> (u32, NonZeroU16, u16, WK) { (self.off, self.len, self.fid, self.kind) }
  
  pub fn kind(&self) -> WK { self.kind }
}

impl Into<Span> for Word {

  fn into(self) -> Span {
    Span::new(self.off, self.len, self.fid)
  }

}
