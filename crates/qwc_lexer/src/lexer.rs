/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::{NonZero, NonZeroU16};

use qwc_arena::File;
use qwc_diagnostic::{Message, Span, Status::{self, Success}, Summary, msg::*, ret_err};

use crate::{WK, Word, wkind::{CHAR_LUT, CharKind}};


pub struct Lexer<'a> {
  fi: &'a File,
  off: usize,
  cache: Option<(Word, usize)>,
  fid: u16,
}

impl<'a> Lexer<'a> {

  pub fn new(fi: &'a File) -> Self {
    Self{fi, off: 0, cache: None, fid: fi.fid()}
  }


  pub fn peek(&mut self, sum: &mut Summary) -> Status<Word> {
    let t = match self.cache {
      Some((w, ..)) => w,
      None => {
        let (w, o) = self.lex(self.off);

        let w = match w {
          Some(v) => v,
          None => ret_err!(sum, Message::fatal(FILE_FINISHED)),
        };

        self.cache = Some((w,o));
        w
      }
    };

    Status::Success(t)
  }

  pub fn peek_safe(&mut self) -> Option<Word> {
    let t = match self.cache {
      Some((w, ..)) => w,
      None => {
        let (w, o) = self.lex(self.off);

        let w = w?;

        self.cache = Some((w,o));
        w
      }
    };

    Some(t)
  }

  pub fn peek_k(&mut self, sum: &mut Summary) -> Status<(WK, Word)> {
    let a = self.peek(sum)?;
    Status::Success((a.kind(), a))
  }


  pub fn get(&mut self, sum: &mut Summary) -> Status<Word> {
    let t = match self.cache {
      None => {
        let (w, o) = self.lex(self.off);
        self.off = o;
        w
      },
      Some((w, o)) => {
        self.off = o;
        self.cache = None;
        Some(w)
      },
    };

    match t {
      Some(r) => Status::Success(r),
      None => ret_err!(sum, Message::fatal(FILE_FINISHED)),
    }
  }

  pub fn get_safe(&mut self) -> Option<Word> {
    match self.cache {
      None => {
        let (w, o) = self.lex(self.off);
        self.off = o;
        w
      },
      Some((w, o)) => {
        self.off = o;
        self.cache = None;
        Some(w)
      },
    }
  }

  pub fn get_k(&mut self, sum: &mut Summary) -> Status<(WK, Word)> {
    let a = self.get(sum)?;
    Status::Success((a.kind(), a))
  }


  pub fn split_token(&mut self, sum: &mut Summary) -> Status<()> {
    let fk = match self.peek(sum)?.kind {
      WK::Caret2 => WK::Caret,
      WK::Amp2 => WK::Amp,
      WK::Gt2 => WK::Gt,

      _ => return Success(())
    };

    let Some(cache) = &mut self.cache else { unreachable!() }; // because self.peek() cached word

    cache.0.kind = fk;
    cache.0.len = unsafe {NonZero::<u16>::new_unchecked(1)};
    cache.1 -= 1;

    Status::Success(())
  }


  pub fn bump(&mut self, sum: &mut Summary) -> Status<()> {
    match self.cache {
      None => {
        let (w, o) = self.lex(self.off);
        self.off = o;
        
        if w.is_none() {
          ret_err!(sum, Message::fatal(FILE_FINISHED))
        } else {
          Status::Success(())
        }
      },
      Some((.., o)) => {
        self.off = o;
        self.cache = None;

        Status::Success(())
      },
    }
  }


  pub fn pos_extend(&self, start: impl Into<Span>) -> Span {
    let start = start.into().to();
    assert_eq!(self.fid, start.2);

    let len = self.off.saturating_sub(start.0 as usize).max(1);
    
    Span::new(
      start.0,
      NonZeroU16::new(self.fit_len(start.0 as usize, len) as u16).unwrap_or(NonZeroU16::MIN),
      self.fid,
    )
  }

  pub fn pos_extend_file(&self) -> Span {
    let len = self.fi.map().len().saturating_sub(8);

    Span::new(
      0,
      NonZeroU16::new(self.fit_len(0, len) as u16).unwrap_or(NonZeroU16::MIN),
      self.fid,
    )
  }


  pub fn fid(&self) -> u16 { self.fid }

  pub fn file(&self) -> &'a File { self.fi }

  pub fn str(&self, w: Word) -> &'a str {
    assert_eq!(self.fid, w.fid);

    str::from_utf8(&self.fi.map()[(w.off as usize)..((w.off as usize)+(w.len.get() as usize))]).unwrap()
  }

  
  #[inline(always)]
  fn kind(c: u8) -> CharKind { CHAR_LUT[c as usize] }

  
  #[inline(always)]
  fn char_at(bytes: &[u8], off: usize) -> Option<(char, usize)> {
    let len = Self::utf8_len(bytes[off]).min(bytes.len() - off);

    str::from_utf8(&bytes[off..off + len]).ok()?.chars().next().map(|c| (c, len))
  }

  #[inline(always)]
  fn start_len(bytes: &[u8], off: usize) -> usize {
    let b = bytes[off];

    if b.is_ascii() { return (b.is_ascii_alphabetic() || b == b'_') as usize }

    match Self::char_at(bytes, off) {
      Some((c, len)) if c == '_' || unicode_ident::is_xid_start(c) => len,
      _ => 0,
    }
  }

  #[inline(always)]
  fn continue_len(bytes: &[u8], off: usize) -> usize {
    let b = bytes[off];

    if b.is_ascii() { return (b.is_ascii_alphanumeric() || b == b'_') as usize }

    match Self::char_at(bytes, off) {
      Some((c, len)) if c == '_' || unicode_ident::is_xid_continue(c) => len,
      _ => 0,
    }
  }

  #[inline(always)]
  fn utf8_len(lead: u8) -> usize {
    match lead {
      0xF0..=0xF7 => 4,
      0xE0..=0xEF => 3,
      0xC0..=0xDF => 2,
      _ => 1,
    }
  }

  #[inline(always)]
  fn fit_len(&self, off: usize, len: usize) -> usize {
    if len <= u16::MAX as usize { return len }

    let map = self.fi.map();
    let mut n = u16::MAX as usize;
    while n > 1 && (map[off + n] & 0xC0) == 0x80 { n -= 1; }
    n
  }

  #[inline(always)]
  fn make(&self, off: usize, len: usize, kind: WK) -> Word {
    let fit = self.fit_len(off, len);
    let kind = if fit < len { WK::Unknown } else { kind };

    Word::new_safe(off, fit, self.fid, kind)
  }

  
  fn lex(&self, mut off: usize) -> (Option<Word>, usize) {
    let size = self.fi.map().len() -8;

    loop {
      macro_rules! get {
        () => {
          match self.fi.map().get(off) {
            Some(val) => *val,
            None => return (None, off),
          }
        };
      }

      macro_rules! geti {
        ($i:expr) => {
          match self.fi.map().get(off + $i) {
            Some(val) => *val,
            None => return (None, off),
          }
        };
      }

      // EOF
      if off >= size { return (None, off); }
      
      let knd = Lexer::kind(get!());

      // String
      match knd {
        CharKind::String => {
          let str_sym = get!();
          let legoff = off;
          off += 1;

          while off < size && get!() != str_sym {
            if get!() == b'\\' && off +1 < size { off += 2; } else { off += 1; }
          }

          let kind = if off < size { off += 1; WK::String } else { WK::Unknown };

          return (Some(self.make(legoff, off - legoff, kind)), off);
        }

        CharKind::Ignored => {
          while off < size && Lexer::kind(get!()) == CharKind::Ignored { off += 1; }
          continue;
        }
      
        CharKind::Symbol => {
          let legoff = off;
  
          let (s, k) = match [geti!(0) as char, geti!(1) as char, geti!(2) as char] {
            // Comment
            ['/','/', _ ]  => {
              while off < size && get!() != b'\n' { off += 1; }
              continue;
            }
            ['/','*', _ ]  => {
              off += 2;
              let mut depth = 1;
              while off < size && depth > 0 {
                match [geti!(0) as char, geti!(1) as char] {
                  ['/','*'] => { depth += 1; off += 2; }
                  ['*','/'] => { depth -= 1; off += 2; }
  
                  [..] => off += 1, 
                }
              }
  
              // An unterminated block comment runs to the end of the file; report it as Unknown
              if depth > 0 {
                return (Some(self.make(legoff, off - legoff, WK::Unknown)), off);
              }
  
              continue;
            }
  
  
            // < >
            ['<','<','='] => (3, WK::Lt2Eq),
            ['<','<', _ ] => (2, WK::Lt2),
            ['<','>', _ ] => (2, WK::LtGt),
            ['<','=','>'] => (3, WK::LtEqGt),
            ['<','=', _ ] => (2, WK::LtEq),
            ['<','-', _ ] => (2, WK::ArrowLeft),
            ['<', _ , _ ] => (1, WK::Lt),
            
            ['>','>','='] => (3, WK::Gt2Eq),
            ['>','>', _ ] => (2, WK::Gt2),
            ['>','<', _ ] => (2, WK::GtLt),
            ['>','=','<'] => (3, WK::GtEqLt),
            ['>','=', _ ] => (2, WK::GtEq),
            ['>', _ , _ ] => (1, WK::Gt),
            
  
            // & | ^
            ['&','&','='] => (3, WK::Amp2Eq),
            ['&','&', _ ] => (2, WK::Amp2),
            ['&', _ , _ ] => (1, WK::Amp),
            
            ['|','|','='] => (3, WK::Pipe2Eq),
            ['|','|', _ ] => (2, WK::Pipe2),
            ['|', _ , _ ] => (1, WK::Pipe),
            
            ['^','^','='] => (3, WK::Caret2Eq),
            ['^','^', _ ] => (2, WK::Caret2),
            ['^', _ , _ ] => (1, WK::Caret),
            
  
            // + - * / %
            ['+','|', _ ] => (2, WK::AddPipe),
            ['+','=', _ ] => (2, WK::AddEq),
            ['+', _ , _ ] => (1, WK::Add),
            
            ['-','|', _ ] => (2, WK::SubPipe),
            ['-','>', _ ] => (2, WK::ArrowRight),
            ['-','=', _ ] => (2, WK::SubEq),
            ['-', _ , _ ] => (1, WK::Sub),
            
            ['*','|', _ ] => (2, WK::MulPipe),
            ['*','=', _ ] => (2, WK::MulEq),
            ['*', _ , _ ] => (1, WK::Mul),
            
            ['/','=', _ ] => (2, WK::DivEq),
            ['/', _ , _ ] => (1, WK::Div),
            
            ['%','=', _ ] => (2, WK::RemEq),
            ['%', _ , _ ] => (1, WK::Rem),
            
  
            // () [] {}
            ['(', _ , _ ] => (1, WK::ParenL),
            [')', _ , _ ] => (1, WK::ParenR),
            ['[', _ , _ ] => (1, WK::BracketL),
            [']', _ , _ ] => (1, WK::BracketR),
            ['{', _ , _ ] => (1, WK::BraceL),
            ['}', _ , _ ] => (1, WK::BraceR),
  
            
            // =
            ['=','=', _ ] => (2, WK::Eq2),
            ['=','>', _ ] => (2, WK::FatArrow),
            ['=', _ , _ ] => (1, WK::Eq),
  
  
            // . , : ;
            ['.','.','.'] => (3, WK::Dot3),
            ['.','.','='] => (3, WK::Dot2Eq),
            ['.','.', _ ] => (2, WK::Dot2),
            ['.', _ , _ ] => (1, WK::Dot),
            [',', _ , _ ] => (1, WK::Comma),
            [':',':', _ ] => (2, WK::Colon2),
            [':', _ , _ ] => (1, WK::Colon),
            [';', _ , _ ] => (1, WK::Semicolon),
  
  
            // !
            ['!','[', _ ] => (2, WK::BangAttr),
            ['!','=', _ ] => (2, WK::BangEq),
            ['!','!', _ ] => (2, WK::Bang2),
            ['!', _ , _ ] => (1, WK::Bang),
  
  
            // #
            ['#','[', _ ] => (2, WK::HashAttr),
            ['#', _ , _ ] => (1, WK::Hash),
  
  
            // @ ? ~ ` _
            ['@', _ , _ ] => (1, WK::At),
            ['?', _ , _ ] => (1, WK::Question),
            ['~', _ , _ ] => (1, WK::Tilde),
            ['`', _ , _ ] => (1, WK::Backtick),
            ['_', _ , _ ] => (1, WK::Underscore),
  
            [..] => (1, WK::Unknown),
          };
  
  
          off += s;
  
          return (Some(self.make(legoff, s, k)), off);
        }

        CharKind::Numeral => {
          let start = off;
          let bytes = &self.fi.map()[..size];

          off += 1;
          let mut has_dot = false;

          while off < size {
            let b = bytes[off];
            let n = Self::continue_len(bytes, off);

            if n > 0 {
              off += n;
            } else if b == b'.' && !has_dot {
              if off +1 < size && bytes[off +1] == b'.' { break }
              has_dot = true;
              off += 1;
            } else {
              break;
            }
          }

          return (Some(self.make(start, off - start, if has_dot {WK::NumberPoint} else {WK::Number})), off);
        }

        CharKind::Word => {
          let start = off;
          let bytes = &self.fi.map()[..size];

          let first = bytes[start];

          if first == b'_' {
            let next_is_word = (start +1 < size) && Self::continue_len(bytes, start +1) > 0;
            if !next_is_word {
              off += 1;
              return (Some(self.make(start, 1, WK::Underscore)), off);
            }
          }

          let first_len = Self::start_len(bytes, start);

          if first_len > 0 {
            off += first_len;

            while off < size {
              let n = Self::continue_len(bytes, off);
              if n == 0 { break }

              off += n;
            }

            let len = off - start;
            let slice = &bytes[start..off];

            let kind = match slice {
              b"if"    => WK::If,
              b"ef"    => WK::Ef,
              b"else"  => WK::Else,
              b"match" => WK::Match,
              b"loop"  => WK::Loop,
              b"while" => WK::While,
              b"for"   => WK::For,
              b"in"    => WK::In,
              b"let"   => WK::Let,
              b"var"   => WK::Var,
              b"fun"   => WK::Fun,
              b"task"  => WK::Task,
              b"init"  => WK::Init,
              b"fini"  => WK::Fini,

              b"using"   => WK::Using,
              b"struct"  => WK::Struct,
              b"iface"   => WK::Iface,
              b"trait"   => WK::Trait,
              b"enum"    => WK::Enum,
              b"variant" => WK::Variant,
              b"flags"   => WK::Flags,
              b"generic" => WK::Generic,
              b"mod"     => WK::Mod,
              b"use"     => WK::Use,
              b"impl"    => WK::Impl,

              b"pub"   => WK::Pub,
              b"priv"  => WK::Priv,
              b"prot"  => WK::Prot,
              b"crate" => WK::Crate,
              b"super" => WK::Super,
              b"mut"   => WK::Mut,
              b"reqs"  => WK::Reqs,

              b"ret"      => WK::Ret,
              b"break"    => WK::Break,
              b"continue" => WK::Continue,
              b"die"      => WK::Die,

              b"unsafe"   => WK::Unsafe,
              b"relaxed"  => WK::Relaxed,

              b"true"  => WK::True,
              b"false" => WK::False,
              b"undef" => WK::Undef,
              b"unreachable" => WK::Unreachable,

              b"self" => WK::SelfS,
              b"Self" => WK::SelfB,
              b"type" => WK::Type,
              b"as"   => WK::As,

              _ => WK::Word,
            };

            return (Some(self.make(start, len, kind)), off);
          }
 
          let len = Self::utf8_len(first).min(size - start);
          off += len;

          return (Some(self.make(start, len, WK::Unknown)), off);
        }
      }
    }
  }

}
