/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{num::NonZero, ops::Range};

use qwc_arena::Files;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
  off: u32,
  len: u16,
  fid: u16,
}

impl Span {
  pub const BUILTIN: Self = Self{off: 0, len: 0, fid: 0};

  pub fn is_builtin(&self) -> bool {
    if self.len != 0 { return false }

    assert_eq!(self.off, 0);
    assert_eq!(self.fid, 0);

    true
  }


  #[cfg(any(feature = "lexer-internal", feature = "unit-internal"))]
  pub fn new(off: u32, len: NonZero<u16>, fid: u16) -> Self {
    Self{ off, len: len.get(), fid }
  }


  #[cfg(any(feature = "lexer-internal", feature = "ast-internal", feature = "unit-internal"))]
  pub fn to(&self) -> (u32, NonZero<u16>, u16) {
    (self.off, NonZero::<u16>::new(self.len).unwrap(), self.fid)
  }


  pub fn str<'a>(&self, far: &'a Files) -> &'a str {
    let rng = (self.off as usize)..((self.off as usize)+(NonZero::<u16>::new(self.len).unwrap().get() as usize));

    let a = &far.get(self.fid).map()[rng];

    str::from_utf8(a).unwrap()
  }

  pub fn interval(&self, far: &Files) -> Range<[NonZero<u32>; 2]> {
    let calc = |text: &[u8], offset: usize| -> [NonZero<u32>; 2] {
      let mut line = 1;
      let mut last_newline_pos = 0;

      for (i, _) in text.iter().enumerate().take(offset) {
        line += 1;
        last_newline_pos = i + 1;
      }

      let mut column = 1;
      for c in text.iter().take(offset).skip(last_newline_pos) {
        if (c & 0xC0) != 0x80 {
          column += 1;
        }
      }

      [NonZero::<u32>::new(line).unwrap(), NonZero::<u32>::new(column).unwrap()]
    };

    let text = &far.get(self.fid).map();
    
    calc(text, self.off as usize)..calc(text, (self.off as usize) + (NonZero::<u16>::new(self.len).unwrap().get() as usize))
  }

  pub fn range(&self) -> Range<usize> {
    (self.off as usize)..((self.off as usize) + (NonZero::<u16>::new(self.len).unwrap().get() as usize))
  }


  pub fn fid(&self) -> u16 {
    self.fid
  }

}
