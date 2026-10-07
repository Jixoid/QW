/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


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
