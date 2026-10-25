/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::fmt;
use std::{collections::HashMap, fs};

use qwc_arena::File;
use qwc_diagnostic::{CodedMsg, Label, Message, Status::{self, Success}, StatusApi2, Summary, msg::*, ret_err, send_err};
use qwc_lexer::{Lexer, WK, Word};


#[derive(Clone, PartialEq)]
pub enum Value {
  Null,
  Int(i64),
  Float(f64),
  Bool(bool),
  Str(String),
  Arr(Vec<Value>),
  Stc(HashMap<String, Value>),
  Tup(Vec<Value>),
}


#[derive(Debug)]
pub struct Error {
  msg: String,
}

impl From<&str> for Error {
  fn from(value: &str) -> Self { Error{ msg: value.to_string() } }
}

impl From<CodedMsg> for Error {
  fn from(value: CodedMsg) -> Self { Error { msg: value.into() } }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.msg)
  }
}



impl Value {
  pub fn make_null() -> Self            { Value::Null }
  pub fn make_int(v: i64) -> Self       { Value::Int(v) }
  pub fn make_float(v: f64) -> Self     { Value::Float(v) }
  pub fn make_bool(v: bool) -> Self     { Value::Bool(v) }
  pub fn make_string(v: String) -> Self { Value::Str(v) }
  pub fn make_struct() -> Self          { Value::Stc(HashMap::new()) }
  pub fn make_array() -> Self           { Value::Arr(vec![]) }
  pub fn make_tuple() -> Self           { Value::Tup(vec![]) }
}

impl Value {

  pub fn push_array(&mut self, v: Value) -> Result<(), Error> {
    match self {
      Self::Arr(subs) => subs.push(v),
      _ => return Err(Error::from(TYPE_NOT_ARRAY)),
    };

    Ok(())
  }

  pub fn push_struct(&mut self, k: &str, v: Value) -> Result<(), Error> {
    match self {
      Self::Stc(subs) => subs.insert(k.to_string(), v),
      _ => return Err(Error::from(TYPE_NOT_STRUCT)),
    };

    Ok(())
  }

}

impl Value {

  pub fn load_file(fi: &File, sum: &mut Summary) -> Status<Value> {
    Parser::parse_root(&mut Lexer::new(fi), sum)
  }

  pub fn save_file(&self, fpath: String) -> Result<(), String> {
    fs::write(fpath, format!("{}", RawWriter(self, 0, true))).map_err(|e| e.to_string())
  }

}


struct RawWriter<'a> (&'a Value, usize, bool /* is first */);

impl<'a> fmt::Display for RawWriter<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let val = self.0;
    let idn = self.1;
    
    match val {
      Value::Null => write!(f, "null")?,

      Value::Int(v) => write!(f, "{}", *v)?,
      Value::Float(v) => write!(f, "{}", *v)?,
      Value::Bool(v) => write!(f, "{}", if *v {"true"} else {"false"})?,
      Value::Str(v) => write!(f, "\"{}\"", *v)?,

      Value::Arr(v) => {
        writeln!(f, "[")?;
        for x in v { writeln!(f, "{}{},", "\t".repeat(idn+1), RawWriter(x, idn+1, false))? }
        write!(f, "{}]", "\t".repeat(idn))?;
      }

      Value::Stc(v) => {
        if self.2 {
          for (s, x) in v { writeln!(f, "{}{}: {},", "\t".repeat(idn), s, RawWriter(x, idn, false))? }
        } else {
          writeln!(f, "{{")?;
          for (s, x) in v { writeln!(f, "{}{}: {},", "\t".repeat(idn+1), s, RawWriter(x, idn+1, false))? }
          write!(f, "{}}}", "\t".repeat(idn))?;
        }
      }
      
      Value::Tup(v) => {
        for x in v { write!(f, "{}", RawWriter(x, idn+1, false))? }
      }
    }

    Ok(())
  }
}



struct Parser;

impl Parser {

  fn unquote(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
      let mut res = String::new();
      let raw = &s[1..s.len()-1];
      let mut chars = raw.chars().peekable();
      while let Some(c) = chars.next() {
        if c == '\\' {
          if let Some(&nc) = chars.peek() {
            match nc {
              'n' => { res.push('\n'); chars.next(); },
              'r' => { res.push('\r'); chars.next(); },
              't' => { res.push('\t'); chars.next(); },
              '\\' => { res.push('\\'); chars.next(); },
              '"' => { res.push('"'); chars.next(); },
              _ => { res.push('\\'); res.push(nc); chars.next(); },
            }
          } else {
            res.push('\\');
          }
        } else {
          res.push(c);
        }
      }
      res
    } else {
      s.to_string()
    }
  }

  fn parse_root(lex: &mut Lexer, sum: &mut Summary,) -> Status<Value> {
    match lex.peek(sum)?.kind() {
      WK::Word => Self::parse_struct(lex, sum, true),

      _ => Self::parse_value(lex, sum),
    }
  }

  fn parse_struct(lex: &mut Lexer, sum: &mut Summary, is_root: bool) -> Status<Value> {
    let mut ctn = HashMap::new();
    
    if !is_root {
      lex.get(sum)?.expected_kind(WK::BraceL, sum)?;
    }

    loop {
      if is_root {
        if lex.peek_safe().is_none() { break }
      } else {
        if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }
      }

      let name = lex.get(sum)?.expected_word(sum)?;
      let name = lex.str(name).to_string();

      lex.get(sum)?.expected_kind(WK::Colon, sum)?;

      let it = Self::parse_value(lex, sum)?;

      ctn.insert(name, it);

      if is_root && let None = lex.peek_safe() { break }

      if let Some(w) = lex.peek_safe() && w.kind() == WK::Comma { lex.bump(sum)? }
    }

    Success(Value::Stc(ctn))
  }

  fn parse_array(lex: &mut Lexer, sum: &mut Summary) -> Status<Value> {
    let mut ctn = vec![];
    lex.get(sum)?.expected_kind(WK::BracketL, sum)?;


    loop {
      if lex.peek(sum)?.kind() == WK::BracketR { lex.bump(sum)?; break }

      let it = Self::parse_value(lex, sum)?;

      ctn.push(it);

      if let Some(w) = lex.peek_safe() && w.kind() == WK::Comma { lex.bump(sum)? }
    }

    Success(Value::Arr(ctn))
  }

  fn parse_value(lex: &mut Lexer, sum: &mut Summary) -> Status<Value> {
    let mut temp_tuple = vec![];
    let mut mask: u8 = 0;

    loop {
      let (c, ma, val) = match lex.peek_k(sum)? {
        (WK::True | WK::False, _) => { let c = lex.get(sum)?; (c, 1, Value::Bool(c.kind() == WK::True))}

        (WK::String, _) => { let c = lex.get(sum)?; (c, 2, Value::Str(Self::unquote(lex.str(c))))}

        (WK::Number | WK::Sub, _) => {
          let c = lex.get(sum)?;
          
          let (c, sig) = if c.kind() == WK::Sub { (lex.get(sum)?.expected_kind(WK::Number, sum)?, false) } else { (c, true) };

          let str = lex.str(c);
          
          let it = match str.contains('.') {
            true  => Value::Float(str.parse::<f64>().map_err(|_| send_err!(sum, Message::error(CANNOT_CONVERT_FLOAT, Label::new_pos(c)))).status()? * (if sig {1.} else {-1.})),
            false => Value::Int(str.parse::<i64>().map_err(|_| send_err!(sum, Message::error(CANNOT_CONVERT_INTEGER, Label::new_pos(c)))).status()? * (if sig {1} else {-1})),
          };

          (c, 4, it)
        }
        
        (WK::BraceL, c) => (c, 8, Self::parse_struct(lex, sum, false)?),
        
        (WK::BracketL, c) => (c, 16, Self::parse_array(lex, sum)?),

        (_, c) => ret_err!(sum, Message::error(INVALID_VALUE.args(&[lex.str(c)]), Label::new(c, VALUE_IS_HERE)))
      };

      temp_tuple.push(val);

      if (mask & ma) != 0 { ret_err!(sum, Message::error(DUPLICATED_VALUE, Label::new_pos(c))) } else { mask |= ma }
      
      match lex.peek_safe().map(|w| w.kind()) {
        Some(WK::True | WK::False | WK::String | WK::BraceL | WK::BracketL | WK::Number) => {}
        _ => break,
      }
    }

    if temp_tuple.len() == 1 {
      Success(temp_tuple.pop().unwrap())
    } else {
      Success(Value::Tup(temp_tuple))
    }
  }

}


trait A {
  fn expected_word(self, sum: &mut Summary) -> Status<Word>;
  fn expected_kind(self, kind: WK, sum: &mut Summary) -> Status<Word>;
}

impl A for Word {

  fn expected_word(self, sum: &mut Summary) -> Status<Self> {
    if self.kind() == WK::Word { Success(self) }
    else { ret_err!(sum, Message::error(EXPECTED_WORD, Label::new_pos(self))) }
  }

  fn expected_kind(self, kind: WK, sum: &mut Summary) -> Status<Self> {
    if self.kind() == kind { Success(self) }
    else {
      ret_err!(sum, Message::error(EXPECTED_KIND2
        .args(&[
          &format!("{:?}", kind),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      ))
    }
  }

}
