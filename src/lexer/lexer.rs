use crate::{control::module::ModuleFile, diagnostic::Message};


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CharKind {
  Ignored = 0x1,
  Symbol  = 0x2,
  Numeral = 0x4,
  String  = 0x8,
  Word    = 0x10,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum WordKind {
    // Bases
    Number,
    String,
    Word,
    EOF,
    Unknown,

    // Shift
    AssignmentLeftShift, // <<=
    AssignmentRighShift, // >>=

    ShiftLeft, // <<
    ShiftRigh, // >>

    // Arithmetic
    AssignmentAdd, // +=
    AssignmentSub, // -=
    AssignmentMul, // *=
    AssignmentDiv, // /=
    AssignmentRem, // %=

    // Logical
    AssignmentLogicalAnd, // &&=
    AssignmentLogicalXor, // ||=
    AssignmentLogicalOr,  // ^^=
    
    LogicalAnd, // &&
    LogicalXor, // ||
    LogicalOr,  // ^^

    // Bitwise
    AssignmentBitwiseAnd, // &=
    AssignmentBitwiseXor, // |=
    AssignmentBitwiseOr,  // ^=
    
    BitwiseAnd, // &
    BitwiseXor, // |
    BitwiseOr,  // ^

    // Brackets
    DoubleSquareBracketBeg, // [[
    DoubleSquareBracketEnd, // ]]

    SquareBracketBeg, // [
    SquareBracketEnd, // ]

    CurlyBracketBeg, // {
    CurlyBracketEnd, // }

    ParenBeg, // (
    ParenEnd, // )

    AngleBeg, // <
    AngleEnd, // >

    // Equalities
    Equal,        // ==
    NotEqual,     // !=
    BiggerEqual,  // >=
    SmallerEqual, // <=

    // Punctuation
    Scope,     // ::
    Colon,     // :
    Semicolon, // ;
    Comma,     // ,
    Dot,       // .
    Hash,      // #
    At,        // @
    Question,  // ?
    Tilde,     // ~
    Backtick,  // `

    // Assignment
    Assign, // =

    // Arithmetic (single)
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Rem, // %

    // Directives / Special
    CompilerDirective, // ![
    Bang,              // !

    ArrowLeft, // <-
    ArrowRigh, // ->
    FatArrow,  // =>

    RotateLeft, // <<|
    RotateRigh, // |>>
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word<'a> {
  pub mol: &'a ModuleFile,
  pub off: usize,
  pub size: usize,
  pub kind: WordKind,
}

impl<'a> Word<'a> {

  pub fn new(mol: &'a ModuleFile, off: usize, size: usize, kind: WordKind) -> Word<'a> { Word{mol, off, size, kind} }

  pub fn str(&self) -> &str { &self.mol.mmap[self.off..(self.off+self.size)] }

  pub fn string(&self) -> String { String::from(self.str()) }


  pub fn interval(&self) -> (HumanPos, HumanPos) {
    let calc = |text: &str, offset: usize| -> HumanPos {
      let mut line = 1;
      let mut last_newline_pos = 0;

      let bytes = text.as_bytes();
      for i in 0..offset {
        if bytes[i] == b'\n' {
          line += 1;
          last_newline_pos = i + 1;
        }
      }

      let mut column = 1;
      for i in last_newline_pos..offset {
        let c = bytes[i];
        if (c & 0xC0) != 0x80 {
          column += 1;
        }
      }

      HumanPos { line, column }
    };

    let text = self.mol.mmap.as_str();
    (calc(text, self.off), calc(text, self.off + self.size))
  }
  
}



pub struct Lexer<'a> {
  pub mol: &'a ModuleFile,
  pub off: usize,
  pub store: Vec<Word<'a>>,
}

impl<'a> Lexer<'a> {

  pub fn new_module(m: &'a ModuleFile) -> Self { return Self{mol: m, off: 0, store: Vec::new()} }

  #[inline(always)]
  pub fn kind(c: u8) -> CharKind { return CHAR_LUT[c as usize] }

  pub fn store(&mut self, w: Word<'a>) {
    self.store.push(w);
  }


  pub fn lex(&mut self) -> Word<'a> {
    if let Some(w) = self.store.pop() {
      return w;
    }

    let size = self.mol.mmap.len();

    loop {
      macro_rules! get {
        () => {
          match self.mol.mmap.as_bytes().get(self.off) {
            Some(val) => *val,
            None => return Word::new(self.mol, 0, 0, WordKind::EOF),
          }
        };
      }

      macro_rules! geti {
        ($i:expr) => {
          match self.mol.mmap.as_bytes().get(self.off + $i) {
            Some(val) => *val,
            None => return Word::new(self.mol, 0, 0, WordKind::EOF),
          }
        };
      }

      
      // EOF
      if self.off >= size {
        return Word::new(self.mol, 0, 0, WordKind::EOF);
      }
      
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
          return Word::new(self.mol, 0, 0, WordKind::EOF);
        }

        return Word::new(self.mol, legoff, self.off - legoff, WordKind::String);
      }

      // Whitespace
      if knd == CharKind::Ignored {
        while self.off < size && Lexer::kind(get!()) == CharKind::Ignored { self.off += 1; }
        continue;
      }

      // Symbols
      if knd == CharKind::Symbol {
        let legoff = self.off;
        
        match get!() {
          b'<' => {
            if self.off + 1 < size {
              if geti!(1) == b'<' {
                if self.off + 2 < size && geti!(2) == b'=' { self.off += 3; return Word::new(self.mol, legoff, 3, WordKind::AssignmentLeftShift); } // "<<="
                if self.off + 2 < size && geti!(2) == b'|' { self.off += 3; return Word::new(self.mol, legoff, 3, WordKind::RotateLeft); } // "<<|"
                self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::ShiftLeft); // "<<"
              }
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::SmallerEqual); } // "<="
              if geti!(1) == b'-' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::ArrowLeft); } // "<-"
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::AngleBeg); // "<"
          }

          b'>' => {
            if self.off + 1 < size {
              if geti!(1) == b'>' {
                if self.off + 2 < size && geti!(2) == b'=' { self.off += 3; return Word::new(self.mol, legoff, 3, WordKind::AssignmentRighShift); } // ">>="
                self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::ShiftRigh); // ">>"
              }
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::BiggerEqual); } // ">="
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::AngleEnd); // ">"
          }

          b'|' => {
            if self.off + 1 < size {
              if geti!(1) == b'>' {
                if self.off + 2 < size && geti!(2) == b'>' { self.off += 3; return Word::new(self.mol, legoff, 3, WordKind::RotateRigh); } // "|>>"
              }
              if geti!(1) == b'|' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::LogicalOr); } // "||"
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentBitwiseOr); } // "|="
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::BitwiseOr); // "|"
          }

          b'-' => {
            if self.off + 1 < size {
              if geti!(1) == b'>' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::ArrowRigh); } // "->"
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentSub); } // "-="
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Sub); // "-"
          }

          b'+' => {
            if self.off + 1 < size && geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentAdd); } // "+="
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Add); // "+"
          }

          b'*' => {
            if self.off + 1 < size && geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentMul); } // "*="
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Mul); // "*"
          }

          b'%' => {
            if self.off + 1 < size && geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentRem); } // "%="
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Rem); // "%"
          }

          b'=' => {
            if self.off + 1 < size {
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::Equal); } // "=="
              if geti!(1) == b'>' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::FatArrow); } // "=>"
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Assign); // "="
          }

          b':' => {
            if self.off + 1 < size && geti!(1) == b':' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::Scope); } // "::"
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Colon); // ":"
          }

          b'/' => {
            if self.off + 1 < size {
              if geti!(1) == b'/' { // Comment
                while self.off < size && get!() != b'\n' { self.off += 1; }
                continue;
              }
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentDiv); } // "/="
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Div); // "/"
          }

          b'!' => {
            if self.off + 1 < size {
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::NotEqual); } // "!="
              if geti!(1) == b'[' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::CompilerDirective); } // "!["
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Bang); // "!"
          }

          b'&' => {
            if self.off + 1 < size {
              if geti!(1) == b'&' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::LogicalAnd); } // "&&"
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentBitwiseAnd); } // "&="
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::BitwiseAnd); // "&"
          }

          b'^' => {
            if self.off + 1 < size {
              if geti!(1) == b'^' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::LogicalXor); } // "^^"
              if geti!(1) == b'=' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::AssignmentBitwiseXor); } // "^="
            }
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::BitwiseXor); // "^"
          }

          b'[' => {
            if self.off + 1 < size && geti!(1) == b'[' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::DoubleSquareBracketBeg); } // "[["
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::SquareBracketBeg); // "["
          }

          b']' => {
            if self.off + 1 < size && geti!(1) == b']' { self.off += 2; return Word::new(self.mol, legoff, 2, WordKind::DoubleSquareBracketEnd); } // "]]"
            self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::SquareBracketEnd); // "]"
          }

          b'{' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::CurlyBracketBeg); } // "{"
          b'}' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::CurlyBracketEnd); } // "}"
          b'(' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::ParenBeg); }       // "("
          b')' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::ParenEnd); }       // ")"
          b';' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Semicolon); }      // ";"
          b',' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Comma); }          // ","
          b'.' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Dot); }            // "."
          b'#' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Hash); }           // "#"
          b'@' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::At); }             // "@"
          b'?' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Question); }       // "?"
          b'~' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Tilde); }          // "~"
          b'`' => { self.off += 1; return Word::new(self.mol, legoff, 1, WordKind::Backtick); }       // "`"

          _ => {
            // Unknown Symbol (fallback)
            self.off += 1;
            return Word::new(self.mol, legoff, 1, WordKind::Unknown);
          }
        }
      }

      // Word / Numeral
      let legoff = self.off;
      let mut is_num = false;
      if self.off < size {
        let ck_first = Lexer::kind(self.mol.mmap.as_bytes()[self.off]) as u8;
        is_num = (ck_first & CharKind::Numeral as u8) != 0;
      }

      while self.off < size {
        let byte = get!();
        let ck = Lexer::kind(byte) as u8;
        if (ck & (CharKind::Word as u8 | CharKind::Numeral as u8)) != 0 {
          self.off += 1;
        } else if is_num && byte == b'.' {
          self.off += 1;
        } else {
          break;
        }
      }

      if legoff == self.off {
        self.off += 1;
        continue;
      }

      let kind = if is_num { WordKind::Number } else { WordKind::Word };
      return Word::new(self.mol, legoff, self.off - legoff, kind);
    }
  }


  pub fn get(&mut self) -> Result<Word<'a>, Message<'a>> {
    let t = self.lex();

    if t.kind == WordKind::EOF {
      Err(Message::fatal(t, String::from("file finished"), Vec::new()))
    } else {
      Ok(t)
    }
  }

}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::control::module::{ModuleFile, ModuleKind};

  #[test]
  fn test_lexer_valid() {
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun main() -> void {}".to_string(),
      kind: ModuleKind::Regular,
    };
    let mut lexer = Lexer::new_module(&mfd);
    
    let t1 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t1.str(), "fun");
    
    let t2 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t2.str(), "main");
    
    let t3 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t3.kind, WordKind::ParenBeg);
    
    let t4 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t4.kind, WordKind::ParenEnd);
    
    let t5 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t5.kind, WordKind::ArrowRigh);
    
    let t6 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t6.str(), "void");
    
    let t7 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t7.kind, WordKind::CurlyBracketBeg);
    
    let t8 = lexer.get().unwrap_or_else(|_| panic!());
    assert_eq!(t8.kind, WordKind::CurlyBracketEnd);
    
    let err = match lexer.get() { Err(e) => e, _ => panic!() }; // EOF
    assert!(err.to_string().contains("file finished"));
  }

  #[test]
  fn test_lexer_eof_error() {
    let mfd = ModuleFile {
      fpath: "test.qw".to_string(),
      mmap: "fun".to_string(),
      kind: ModuleKind::Regular,
    };
    let mut lexer = Lexer::new_module(&mfd);
    
    lexer.get().unwrap_or_else(|_| panic!()); // fun
    let err = match lexer.get() { Err(e) => e, _ => panic!() }; // EOF
    assert!(err.to_string().contains("file finished"));
  }
}

