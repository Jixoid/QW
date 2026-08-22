use crate::{ast::Module, diagnostic::Message, route::build::FileArena};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CharKind {
  Ignored = 0x1,
  Symbol  = 0x2,
  Numeral = 0x4,
  String  = 0x8,
  Word    = 0x10,
}


pub type WK = WordKind;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WordKind {
  // Bases
  Number,
  String,
  Word,
  EOF,
  Unknown,

  // Shift

  /// <<=
  AssignmentLeftShift,
  /// >>=
  AssignmentRighShift,

  /// <<
  ShiftLeft,
  /// >>
  ShiftRigh,

  // Arithmetic

  /// +=
  AssignmentAdd,
  /// -=
  AssignmentSub,
  /// *=
  AssignmentMul,
  /// /=
  AssignmentDiv,
  /// %=
  AssignmentRem,

  // Logical

  /// &&=
  AssignmentLogicalAnd,
  /// ^^=
  AssignmentLogicalXor,
  /// ||=
  AssignmentLogicalOr,
  
  /// &&
  LogicalAnd,
  /// ^^
  LogicalXor,
  /// ||
  LogicalOr,

  // Bitwise

  /// &=
  AssignmentBitwiseAnd,
  /// ^=
  AssignmentBitwiseXor,
  /// |=
  AssignmentBitwiseOr,
  
  /// &
  BitwiseAnd,
  /// ^
  BitwiseXor,
  /// |
  BitwiseOr,

  // Brackets

  /// [
  SquareBracketBeg,
  /// ]
  SquareBracketEnd,

  /// {
  CurlyBracketBeg,
  /// }
  CurlyBracketEnd,

  /// (
  ParenBeg,
  /// )
  ParenEnd,

  /// <
  AngleBeg,
  /// >
  AngleEnd,

  // Equalities

  /// ==
  Equal,
  /// !=
  NotEqual,
  /// >=
  BiggerEqual,
  /// <=
  SmallerEqual,

  // Punctuation

  /// ::
  Scope,
  /// :
  Colon,
  /// ;
  Semicolon,
  /// ,
  Comma,
  /// .
  Dot,
  /// ..
  Dot2,
  /// #
  Hash,
  /// @
  At,
  /// ?
  Question,
  /// ~
  Tilde,
  /// `
  Backtick,

  // Assignment

  /// =
  Assign,

  // Arithmetic (single)

  /// +
  Add,
  /// -
  Sub,
  /// *
  Mul,
  /// /
  Div,
  /// %
  Rem,

  // Directives / Special

  /// #[
  Directive,
  /// ![
  Attribute,
  /// !
  Bang,

  /// _
  Underscore,

  /// <-
  ArrowLeft,
  /// ->
  ArrowRigh,
  /// =>
  FatArrow,

  /// <<|
  RotateLeft,
  /// |>>
  RotateRigh,

  // Keyword
  If, Ef, Else, Match, Loop, While, For, In, Let, Var,
  Using, Struct, Iface, Trait, Enum, Flags, Fun, Init, Fini, Generic, Mod, Use, Impl,
  Pub, Priv, Prot, Crate, Super, Mut, Imm,
  Ret, Break, Continue, Die,
}

const fn create_char_lut() -> [CharKind; 256] {
  let mut lut = [CharKind::Word; 256];

  // Whitespace
  let whitespace = [b' ', b'\n', b'\r', b'\t'];
  let mut i = 0;
  while i < whitespace.len() {
    lut[whitespace[i] as usize] = CharKind::Ignored;
    i += 1;
  }

  // Numeral ('0'..='9')
  let mut c = b'0';
  while c <= b'9' {
    lut[c as usize] = CharKind::Numeral;
    c += 1;
  }

  // String
  lut[b'\'' as usize] = CharKind::String;
  lut[b'"' as usize] = CharKind::String;

  // Symbols
  let symbols = [
    b'#', b'{', b'}', b'.', b':', b';', b',', b'=', b'(', b')', 
    b'<', b'>', b'[', b']', b'-', b'+', b'/', b'%', b'*', b'^', 
    b'~', b'&', b'|', b'@', b'?', b'!', b'`',
  ];
  let mut j = 0;
  while j < symbols.len() {
    lut[symbols[j] as usize] = CharKind::Symbol;
    j += 1;
  }

  lut
}

static CHAR_LUT: [CharKind; 256] = create_char_lut();


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HumanPos {
  pub line: usize,
  pub column: usize,
}

#[derive(Clone, Copy)]
pub struct Word {
  pub off: u32,
  pub size: u16,
  pub fid: u16,
  pub kind: WordKind,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
  pub off: u32,
  pub size: u16,
  pub fid: u16,
}


impl<'a> Word {

  pub fn new(off: usize, size: usize, fid: u16, kind: WordKind) -> Word {
    assert!(off  < 0xFFFF_FFFF);
    assert!(size < 0xFFFF);

    Word{off: off as u32, size: size as u16, fid, kind}
  }


  pub fn save(&self) -> Span {
    Span{ off: self.off, size: self.size, fid: self.fid }
  }


  pub fn str(&self, farena: &'a FileArena) -> &'a str {
    let rng = (self.off as usize)..((self.off as usize)+(self.size as usize));

    let a = &farena.get(self.fid).mmap[rng];

    unsafe { str::from_utf8_unchecked(a) }
  }

  pub fn string(&self, farena: &'a FileArena) -> String {
    String::from(self.str(farena))
  }


  pub fn mol(&self, far: &'a FileArena) -> &'a Module {
    far.get(self.fid)
  }

  pub fn interval(&self, farena: &'a FileArena) -> (HumanPos, HumanPos) {
    let calc = |text: &[u8], offset: usize| -> HumanPos {
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

      HumanPos { line, column }
    };

    let text = &farena.get(self.fid).mmap[..];
    (calc(text, self.off as usize), calc(text, (self.off as usize) + (self.size as usize)))
  }
  
}

impl<'a> Span {

  pub fn str(&self, farena: &'a FileArena) -> &'a str {
    let rng = (self.off as usize)..((self.off as usize)+(self.size as usize));

    let a = &farena.get(self.fid).mmap[rng];

    unsafe { str::from_utf8_unchecked(a) }
  }

  pub fn string(&self, farena: &'a FileArena) -> String {
    String::from(self.str(farena))
  }


  pub fn mol(&self, far: &'a FileArena) -> &'a Module {
    far.get(self.fid)
  }

  pub fn interval(&self, farena: &'a FileArena) -> (HumanPos, HumanPos) {
    let calc = |text: &[u8], offset: usize| -> HumanPos {
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

      HumanPos { line, column }
    };

    let text = &farena.get(self.fid).mmap[..];
    (calc(text, self.off as usize), calc(text, (self.off as usize) + (self.size as usize)))
  }
  
}



pub struct Lexer<'a> {
  pub mol: &'a Module,
  pub off: usize,
  pub store: Vec<Word>,
  pub fid: u16,
}

impl<'a> Lexer<'a> {

  pub fn new(m: &'a Module) -> Self {
    return Self{mol: m, off: 0, store: Vec::new(), fid: m.fid}
  }

  pub fn store(&mut self, w: Word) {
    self.store.push(w);
  }


  #[inline(always)]
  pub fn kind(c: u8) -> CharKind { return CHAR_LUT[c as usize] }

  #[inline(always)]
  fn is_word_start(b: u8) -> bool { b.is_ascii_alphabetic() || b == b'_' }

  #[inline(always)]
  fn is_word_continue(b: u8) -> bool { b.is_ascii_alphanumeric() || b == b'_' }

  
  pub fn lex(&mut self) -> Option<Word> {
    if let Some(w) = self.store.pop() {
      return Some(w);
    }

    let size = self.mol.mmap.len() -8;

    loop {
      macro_rules! get {
        () => {
          match self.mol.mmap.get(self.off) {
            Some(val) => *val,
            None => return None,
          }
        };
      }

      macro_rules! geti {
        ($i:expr) => {
          match self.mol.mmap.get(self.off + $i) {
            Some(val) => *val,
            None => return None,
          }
        };
      }

      
      // EOF
      if self.off >= size { return None; }
      
      let knd = Lexer::kind(get!());

      // String
      if knd == CharKind::String {
        let str_sym = get!();
        let legoff = self.off;
        self.off += 1;

        while self.off < size && get!() != str_sym {
          if get!() == b'\\' && self.off + 1 < size {
            self.off += 2;
          } else {
            self.off += 1;
          }
        }

        if self.off < size {
          self.off += 1;
        } else {
          return None;
        }

        return Some(Word::new(legoff, self.off - legoff, self.fid, WordKind::String));
      }

      // Whitespace
      if knd == CharKind::Ignored {
        while self.off < size && Lexer::kind(get!()) == CharKind::Ignored { self.off += 1; }
        continue;
      }

      // Symbols
      if knd == CharKind::Symbol {
        let legoff = self.off;

        let (s,k) = match [geti!(0), geti!(1), geti!(2)] {
          [b'<',b'<',b'='] => { self.off += 3; (3, WordKind::AssignmentLeftShift) } // "<<="
          [b'<',b'<',b'|'] => { self.off += 3; (3, WordKind::RotateLeft) } // "<<|"
          [b'<',b'<', ..]  => { self.off += 2; (2, WordKind::ShiftLeft) } // "<<" 
          [b'<',b'=', ..]  => { self.off += 2; (2, WordKind::SmallerEqual) } // "<="
          [b'<',b'-', ..]  => { self.off += 2; (2, WordKind::ArrowLeft) } // "<-"
          [b'<',..]        => { self.off += 1; (1, WordKind::AngleBeg) } // "<"
          
          [b'>',b'>',b'='] => { self.off += 3; (3, WordKind::AssignmentRighShift) } // ">>="
          [b'>',b'>', ..]  => { self.off += 2; (2, WordKind::ShiftRigh) } // ">>"
          [b'>',b'=', ..]  => { self.off += 2; (2, WordKind::BiggerEqual) } // ">="
          [b'>',..]        => { self.off += 1; (1, WordKind::AngleEnd) } // ">"

          [b'|',b'>',b'>'] => { self.off += 3; (3, WordKind::RotateRigh) } // "|>>"
          [b'|',b'|', ..]  => { self.off += 2; (2, WordKind::LogicalOr) } // "||"
          [b'|',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentBitwiseOr) } // "|="
          [b'|',..]        => { self.off += 1; (1, WordKind::BitwiseOr) } // "|"

          [b'-',b'>', ..]  => { self.off += 2; (2, WordKind::ArrowRigh) } // "->"
          [b'-',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentSub) } // "-="
          [b'-',..]        => { self.off += 1; (1, WordKind::Sub) } // "-"

          [b'+',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentAdd) } // "+="
          [b'+',..]        => { self.off += 1; (1, WordKind::Add) } // "+"

          [b'*',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentMul) } // "*="
          [b'*',..]        => { self.off += 1; (1, WordKind::Mul) } // "*"

          [b'%',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentRem) } // "%="
          [b'%',..]        => { self.off += 1; (1, WordKind::Rem) } // "%"

          [b'=',b'=', ..]  => { self.off += 2; (2, WordKind::Equal) } // "=="
          [b'=',b'>', ..]  => { self.off += 2; (2, WordKind::FatArrow) } // "=>"
          [b'=',..]        => { self.off += 1; (1, WordKind::Assign) } // "="

          [b':',b':', ..]  => { self.off += 2; (2, WordKind::Scope) } // "::"
          [b':',..]        => { self.off += 1; (1, WordKind::Colon) } // ":"

          [b'/',b'/', ..]  => { // Comment
            while self.off < size && get!() != b'\n' { self.off += 1; }
            continue;
          }
          [b'/',b'*', ..]  => { // Comment
            self.off += 2;
            let mut depth = 1;
            while self.off < size && depth > 0 {
              if geti!(0) == b'/' && geti!(1) == b'*' {
                depth += 1;
                self.off += 2;
              } else if geti!(0) == b'*' && geti!(1) == b'/' {
                depth -= 1;
                self.off += 2;
              } else {
                self.off += 1;
              }
            }
            continue;
          }
          [b'/',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentDiv) } // "/="
          [b'/',..]        => { self.off += 1; (1, WordKind::Div) } // "/"

          [b'!',b'=', ..]  => { self.off += 2; (2, WordKind::NotEqual) } // "!="
          [b'!',b'[', ..]  => { self.off += 2; (2, WordKind::Attribute) } // "!["
          [b'!',..]        => { self.off += 1; (1, WordKind::Bang) } // "!"

          [b'#',b'[', ..]  => { self.off += 2; (2, WordKind::Directive) } // "#["
          [b'#',..]        => { self.off += 1; (1, WordKind::Hash) } // "#"

          [b'.',b'.', ..]  => { self.off += 2; (2, WordKind::Dot2) } // ".."
          [b'.',..]        => { self.off += 1; (1, WordKind::Dot) } // "."

          [b'&',b'&', ..]  => { self.off += 2; (2, WordKind::LogicalAnd) } // "&&"
          [b'&',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentBitwiseAnd) } // "&="
          [b'&',..]        => { self.off += 1; (1, WordKind::BitwiseAnd) } // "&"

          [b'^',b'^', ..]  => { self.off += 2; (2, WordKind::LogicalXor) } // "^^"
          [b'^',b'=', ..]  => { self.off += 2; (2, WordKind::AssignmentBitwiseXor) } // "^="
          [b'^',..]        => { self.off += 1; (1, WordKind::BitwiseXor) } // "^"

          [b'[',..] => { self.off += 1; (1, WordKind::SquareBracketBeg) } // "["
          [b']',..] => { self.off += 1; (1, WordKind::SquareBracketEnd) } // "]"
          [b'{',..] => { self.off += 1; (1, WordKind::CurlyBracketBeg) }  // "{"
          [b'}',..] => { self.off += 1; (1, WordKind::CurlyBracketEnd) }  // "}"
          [b'(',..] => { self.off += 1; (1, WordKind::ParenBeg) }         // "("
          [b')',..] => { self.off += 1; (1, WordKind::ParenEnd) }         // ")"
          [b';',..] => { self.off += 1; (1, WordKind::Semicolon) } // ";"
          [b',',..] => { self.off += 1; (1, WordKind::Comma) }     // ","
          [b'@',..] => { self.off += 1; (1, WordKind::At) }        // "@"
          [b'?',..] => { self.off += 1; (1, WordKind::Question) }  // "?"
          [b'~',..] => { self.off += 1; (1, WordKind::Tilde) }     // "~"
          [b'`',..] => { self.off += 1; (1, WordKind::Backtick) }  // "`"

          [..] => { self.off += 1; (1, WordKind::Unknown) }
        };

        return Some(Word::new(legoff, s, self.fid, k));
      }

      // Word / Numeral
      let start = self.off;
      let bytes = &self.mol.mmap[..];
      let size = bytes.len();

      if start >= size { return None; }

      let first = bytes[start];

      if first == b'_' {
        let next_is_word = (start + 1 < size) && Self::is_word_continue(bytes[start + 1]);
        if !next_is_word {
          self.off += 1;
          return Some(Word::new(start, 1, self.fid, WordKind::Underscore));
        }
      }

      if first.is_ascii_digit() {
        self.off += 1;
        let mut has_dot = false;

        while self.off < size {
          let b = bytes[self.off];
          if b.is_ascii_digit() {
            self.off += 1;
          } else if b == b'.' && !has_dot {
            if self.off + 1 < size && bytes[self.off + 1] == b'.' {
              break;
            }
            has_dot = true;
            self.off += 1;
          } else {
            break;
          }
        }

        return Some(Word::new(start, self.off - start, self.fid, WordKind::Number));
      }

      if Self::is_word_start(first) {
        self.off += 1;
        while self.off < size && Self::is_word_continue(bytes[self.off]) {
          self.off += 1;
        }

        let len = self.off - start;
        let slice = &bytes[start..self.off];

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
          b"imm"   => WK::Imm,

          b"ret"      => WK::Ret,
          b"break"    => WK::Break,
          b"continue" => WK::Continue,
          b"die"      => WK::Die,

          _ => WordKind::Word,
        };

        return Some(Word::new(start, len, self.fid, kind));
      }
    }
  }


  pub fn get(&mut self) -> Result<Word, Message> {
    let t = self.lex();

    match t {
      Some(r) => Ok(r),
      None => Err(Message::fatal(Span{off: 0, size: 0, fid: 0}, "file finished", vec![])),
    }
  }

}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::Module;

  fn create_module(code: &str) -> Module {
    let mut mmap = code.as_bytes().to_vec();
    mmap.resize(mmap.len() + 8, 0);
    Module {
      fpath: "test.qw".to_string(),
      fid: 0,
      name: "test".to_string(),
      mmap,
    }
  }

  #[test]
  fn test_multiline_comment() {
    let mol = create_module("let x /* comment */ = 5;");
    let mut lexer = Lexer::new(&mol);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Let);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Word); // x
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Assign);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Number); // 5
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Semicolon);
    assert!(lexer.lex().is_none());
  }

  #[test]
  fn test_nested_multiline_comment() {
    let mol = create_module("/* outer /* inner */ outer */ fun main() {}");
    let mut lexer = Lexer::new(&mol);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Fun);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::Word); // main
    assert_eq!(lexer.lex().unwrap().kind, WordKind::ParenBeg);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::ParenEnd);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::CurlyBracketBeg);
    assert_eq!(lexer.lex().unwrap().kind, WordKind::CurlyBracketEnd);
    assert!(lexer.lex().is_none());
  }
}
