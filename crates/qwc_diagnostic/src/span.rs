use std::{num::{NonZeroU16, NonZeroU32}, ops::Range};

use qwc_arena::Files;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
  pub(crate) off: u32,
  pub(crate) len: NonZeroU16,
  pub(crate) fid: u16,
}

impl Span {

  #[cfg(feature = "lexer-internal")]
  pub fn new(off: u32, len: NonZeroU16, fid: u16) -> Self {
    Self{ off, len, fid }
  }


  #[cfg(feature = "lexer-internal")]
  #[cfg(feature = "ast-internal")]
  pub fn to(&self) -> (u32, NonZeroU16, u16) { (self.off, self.len, self.fid) }


  pub fn str<'a>(&self, far: &'a Files) -> &'a str {
    let rng = (self.off as usize)..((self.off as usize)+(self.len.get() as usize));

    let a = &far.get(self.fid).map()[rng];

    str::from_utf8(a).unwrap()
  }

  pub fn interval<'a>(&self, far: &'a Files) -> Range<[NonZeroU32; 2]> {
    let calc = |text: &[u8], offset: usize| -> [NonZeroU32; 2] {
      let mut line = 1;
      let mut last_newline_pos = 0;

      for i in 0..offset {
        if text[i] == b'\n' {
          line += 1;
          last_newline_pos = i + 1;
        }
      }

      let mut column = 1;
      for i in last_newline_pos..offset {
        let c = text[i];
        if (c & 0xC0) != 0x80 {
          column += 1;
        }
      }

      [NonZeroU32::new(line).unwrap(), NonZeroU32::new(column).unwrap()]
    };

    let text = &far.get(self.fid).map()[..];
    
    calc(text, self.off as usize)..calc(text, (self.off as usize) + (self.len.get() as usize))
  }

  pub fn range(&self) -> Range<usize> {
    (self.off as usize)..((self.off as usize) + (self.len.get() as usize))
  }


  pub fn fid(&self) -> u16 {
    self.fid
  }

}
