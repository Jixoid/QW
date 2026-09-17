use std::num::NonZeroU16;

use qwc_arena::File;
use qwc_diagnostic::{Message, Span, msg::*};

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


  pub fn peek(&mut self) -> Result<Word, Message> {
    let t = match self.cache {
      Some((w, ..)) => w,
      None => {
        let (w, o) = self.lex(self.off);

        let w = match w {
          Some(v) => v,
          None => return Err(Message::fatal(FILE_FINISHED)),
        };

        self.cache = Some((w,o));
        w
      }
    };

    Ok(t)
  }

  pub fn peek_safe(&mut self) -> Option<Word> {
    let t = match self.cache {
      Some((w, ..)) => w,
      None => {
        let (w, o) = self.lex(self.off);

        let w = match w {
          Some(v) => v,
          None => return None,
        };

        self.cache = Some((w,o));
        w
      }
    };

    Some(t)
  }

  pub fn peek_k(&mut self) -> Result<(WK, Word), Message> {
    let a = self.peek()?;
    Ok((a.kind(), a))
  }


  pub fn get(&mut self) -> Result<Word, Message> {
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
      Some(r) => Ok(r),
      None => Err(Message::fatal(FILE_FINISHED)),
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

  pub fn get_k(&mut self) -> Result<(WK, Word), Message> {
    let a = self.get()?;
    Ok((a.kind(), a))
  }


  pub fn bump(&mut self) -> Result<(), Message> {
    match self.cache {
      None => {
        let (w, o) = self.lex(self.off);
        self.off = o;
        
        if w.is_none() {
          Err(Message::fatal(FILE_FINISHED))
        } else {
          Ok(())
        }
      },
      Some((.., o)) => {
        self.off = o;
        self.cache = None;

        Ok(())
      },
    }
  }


  pub fn pos_extend(&self, start: impl Into<Span>) -> Span {
    let start = start.into().to();
    assert_eq!(self.fid, start.2);

    let len = self.off.saturating_sub(start.0 as usize).max(1);
    
    Span::new(
      start.0,
      NonZeroU16::new(len as u16).unwrap_or(NonZeroU16::MIN),
      self.fid,
    )
  }

  pub fn pos_extend_file(&self) -> Span {
    Span::new(
      0,
      
      NonZeroU16::new((self.fi.map().len() -8) as u16).unwrap_or(NonZeroU16::MIN),
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
  fn kind(c: u8) -> CharKind { return CHAR_LUT[c as usize] }

  #[inline(always)]
  fn is_word_start(b: u8) -> bool { b.is_ascii_alphabetic() || b == b'_' }

  #[inline(always)]
  fn is_word_continue(b: u8) -> bool { b.is_ascii_alphanumeric() || b == b'_' }

  
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
      if knd == CharKind::String {
        let str_sym = get!();
        let legoff = off;
        off += 1;

        while off < size && get!() != str_sym {
          if get!() == b'\\' && off + 1 < size {
            off += 2;
          } else {
            off += 1;
          }
        }

        if off < size {
          off += 1;
        } else {
          return (None, off);
        }

        return (Some(Word::new_safe(legoff, off - legoff, self.fid, WK::String)), off);
      }

      // Whitespace
      if knd == CharKind::Ignored {
        while off < size && Lexer::kind(get!()) == CharKind::Ignored { off += 1; }
        continue;
      }

      // Symbols
      if knd == CharKind::Symbol {
        let legoff = off;

        let (s,k) = match [geti!(0), geti!(1), geti!(2)] {
          [b'<',b'<',b'='] => { off += 3; (3, WK::AssignmentLeftShift) } // "<<="
          [b'<',b'<',b'|'] => { off += 3; (3, WK::RotateLeft) } // "<<|"
          [b'<',b'<', ..]  => { off += 2; (2, WK::ShiftLeft) } // "<<" 
          [b'<',b'=', ..]  => { off += 2; (2, WK::SmallerEqual) } // "<="
          [b'<',b'-', ..]  => { off += 2; (2, WK::ArrowLeft) } // "<-"
          [b'<',..]        => { off += 1; (1, WK::AngleBeg) } // "<"
          
          [b'>',b'>',b'='] => { off += 3; (3, WK::AssignmentRighShift) } // ">>="
          [b'>',b'>', ..]  => { off += 2; (2, WK::ShiftRigh) } // ">>"
          [b'>',b'=', ..]  => { off += 2; (2, WK::BiggerEqual) } // ">="
          [b'>',..]        => { off += 1; (1, WK::AngleEnd) } // ">"

          [b'|',b'>',b'>'] => { off += 3; (3, WK::RotateRigh) } // "|>>"
          [b'|',b'|', ..]  => { off += 2; (2, WK::LogicalOr) } // "||"
          [b'|',b'=', ..]  => { off += 2; (2, WK::AssignmentBitwiseOr) } // "|="
          [b'|',..]        => { off += 1; (1, WK::BitwiseOr) } // "|"

          [b'-',b'>', ..]  => { off += 2; (2, WK::ArrowRigh) } // "->"
          [b'-',b'=', ..]  => { off += 2; (2, WK::AssignmentSub) } // "-="
          [b'-',..]        => { off += 1; (1, WK::Sub) } // "-"

          [b'+',b'=', ..]  => { off += 2; (2, WK::AssignmentAdd) } // "+="
          [b'+',..]        => { off += 1; (1, WK::Add) } // "+"

          [b'*',b'=', ..]  => { off += 2; (2, WK::AssignmentMul) } // "*="
          [b'*',..]        => { off += 1; (1, WK::Mul) } // "*"

          [b'%',b'=', ..]  => { off += 2; (2, WK::AssignmentRem) } // "%="
          [b'%',..]        => { off += 1; (1, WK::Rem) } // "%"

          [b'=',b'=', ..]  => { off += 2; (2, WK::Equal) } // "=="
          [b'=',b'>', ..]  => { off += 2; (2, WK::FatArrow) } // "=>"
          [b'=',..]        => { off += 1; (1, WK::Assign) } // "="

          [b':',b':', ..]  => { off += 2; (2, WK::Scope) } // "::"
          [b':',..]        => { off += 1; (1, WK::Colon) } // ":"

          [b'/',b'/', ..]  => { // Comment
            while off < size && get!() != b'\n' { off += 1; }
            continue;
          }
          [b'/',b'*', ..]  => { // Comment
            off += 2;
            let mut depth = 1;
            while off < size && depth > 0 {
              if geti!(0) == b'/' && geti!(1) == b'*' {
                depth += 1;
                off += 2;
              } else if geti!(0) == b'*' && geti!(1) == b'/' {
                depth -= 1;
                off += 2;
              } else {
                off += 1;
              }
            }
            continue;
          }
          [b'/',b'=', ..]  => { off += 2; (2, WK::AssignmentDiv) } // "/="
          [b'/',..]        => { off += 1; (1, WK::Div) } // "/"

          [b'!',b'=', ..]  => { off += 2; (2, WK::NotEqual) } // "!="
          [b'!',b'[', ..]  => { off += 2; (2, WK::Attribute) } // "!["
          [b'!',..]        => { off += 1; (1, WK::Bang) } // "!"

          [b'#',b'[', ..]  => { off += 2; (2, WK::Directive) } // "#["
          [b'#',..]        => { off += 1; (1, WK::Hash) } // "#"

          [b'.',b'.', ..]  => { off += 2; (2, WK::Dot2) } // ".."
          [b'.',..]        => { off += 1; (1, WK::Dot) } // "."

          [b'&',b'&', ..]  => { off += 2; (2, WK::LogicalAnd) } // "&&"
          [b'&',b'=', ..]  => { off += 2; (2, WK::AssignmentBitwiseAnd) } // "&="
          [b'&',..]        => { off += 1; (1, WK::BitwiseAnd) } // "&"

          [b'^',b'^', ..]  => { off += 2; (2, WK::LogicalXor) } // "^^"
          [b'^',b'=', ..]  => { off += 2; (2, WK::AssignmentBitwiseXor) } // "^="
          [b'^',..]        => { off += 1; (1, WK::BitwiseXor) } // "^"

          [b'[',..] => { off += 1; (1, WK::SquareBracketBeg) } // "["
          [b']',..] => { off += 1; (1, WK::SquareBracketEnd) } // "]"
          [b'{',..] => { off += 1; (1, WK::CurlyBracketBeg) }  // "{"
          [b'}',..] => { off += 1; (1, WK::CurlyBracketEnd) }  // "}"
          [b'(',..] => { off += 1; (1, WK::ParenBeg) }         // "("
          [b')',..] => { off += 1; (1, WK::ParenEnd) }         // ")"
          [b';',..] => { off += 1; (1, WK::Semicolon) } // ";"
          [b',',..] => { off += 1; (1, WK::Comma) }     // ","
          [b'@',..] => { off += 1; (1, WK::At) }        // "@"
          [b'?',..] => { off += 1; (1, WK::Question) }  // "?"
          [b'~',..] => { off += 1; (1, WK::Tilde) }     // "~"
          [b'`',..] => { off += 1; (1, WK::Backtick) }  // "`"

          [..] => { off += 1; (1, WK::Unknown) }
        };

        return (Some(Word::new_safe(legoff, s, self.fid, k)), off);
      }

      // Word / Numeral
      let start = off;
      let bytes = &self.fi.map()[..];
      let size = bytes.len();

      if start >= size { return (None, off); }

      let first = bytes[start];

      if first == b'_' {
        let next_is_word = (start + 1 < size) && Self::is_word_continue(bytes[start + 1]);
        if !next_is_word {
          off += 1;
          return (Some(Word::new_safe(start, 1, self.fid, WK::Underscore)), off);
        }
      }

      if first.is_ascii_digit() {
        off += 1;
        let mut has_dot = false;

        while off < size {
          let b = bytes[off];
          if Self::is_word_continue(b) {
            off += 1;
          } else if b == b'.' && !has_dot {
            if off + 1 < size && bytes[off + 1] == b'.' {
              break;
            }
            has_dot = true;
            off += 1;
          } else {
            break;
          }
        }

        return (Some(Word::new_safe(start, off - start, self.fid, WK::Number)), off);
      }

      if Self::is_word_start(first) {
        off += 1;
        while off < size && Self::is_word_continue(bytes[off]) {
          off += 1;
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

          _ => WK::Word,
        };

        return (Some(Word::new_safe(start, len, self.fid, kind)), off);
      }
    }
  }

}
