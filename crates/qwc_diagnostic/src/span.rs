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

      for (i, &c) in text.iter().enumerate().take(offset) {
        if c == b'\n' {
          line += 1;
          last_newline_pos = i + 1;
        }
      }

      let mut column = 1;
      for &c in text.iter().take(offset).skip(last_newline_pos) {
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

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::Path;

  #[test]
  fn test_span_interval() {
    let mut files = Files::default();
    let content = b"hello\nworld\nfoo bar";
    let fid = files.add_buffer(Path::new("test.qw"), content.to_vec());

    // "hello" on line 1, col 1..6
    let span1 = Span { off: 0, len: 5, fid };
    let int1 = span1.interval(&files);
    assert_eq!(int1.start, [NonZero::new(1).unwrap(), NonZero::new(1).unwrap()]);
    assert_eq!(int1.end, [NonZero::new(1).unwrap(), NonZero::new(6).unwrap()]);

    // "world" on line 2, col 1..6 (offset 6..11)
    let span2 = Span { off: 6, len: 5, fid };
    let int2 = span2.interval(&files);
    assert_eq!(int2.start, [NonZero::new(2).unwrap(), NonZero::new(1).unwrap()]);
    assert_eq!(int2.end, [NonZero::new(2).unwrap(), NonZero::new(6).unwrap()]);

    // "bar" on line 3, col 5..8 (offset 16..19)
    let span3 = Span { off: 16, len: 3, fid };
    let int3 = span3.interval(&files);
    assert_eq!(int3.start, [NonZero::new(3).unwrap(), NonZero::new(5).unwrap()]);
    assert_eq!(int3.end, [NonZero::new(3).unwrap(), NonZero::new(8).unwrap()]);
  }

  #[test]
  fn test_span_interval_utf8() {
    let mut files = Files::default();
    let content = "ağaç\n🚀 roket".as_bytes();
    let fid = files.add_buffer(Path::new("test.qw"), content.to_vec());

    // "ağaç" has 4 chars, 6 bytes (a:1, ğ:2, a:1, ç:2)
    let span1 = Span { off: 0, len: 6, fid };
    let int1 = span1.interval(&files);
    assert_eq!(int1.start, [NonZero::new(1).unwrap(), NonZero::new(1).unwrap()]);
    assert_eq!(int1.end, [NonZero::new(1).unwrap(), NonZero::new(5).unwrap()]);

    // Line 2: 🚀 (4 bytes), space (1 byte), roket (5 bytes)
    // "roket" starts at offset 7+4+1 = 12
    let span2 = Span { off: 12, len: 5, fid };
    let int2 = span2.interval(&files);
    // col 1: 🚀, col 2: ' ', col 3: 'r'
    assert_eq!(int2.start, [NonZero::new(2).unwrap(), NonZero::new(3).unwrap()]);
    assert_eq!(int2.end, [NonZero::new(2).unwrap(), NonZero::new(8).unwrap()]);
  }
}

