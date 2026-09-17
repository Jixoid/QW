use core::fmt;
use std::{collections::HashMap, fs};

use qwc_arena::File;
use qwc_diagnostic::{CodedMsg, Label, Message};
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

const TYPE_NOT_ARRAY: CodedMsg = CodedMsg::new_str("type is not a array");
const TYPE_NOT_STRUCT: CodedMsg = CodedMsg::new_str("type is not a struct");
const EXPECTED_WORD: CodedMsg = CodedMsg::new_str("expected word");
const EXPECTED_KIND: CodedMsg = CodedMsg::new_str("expected `{}`, but found `{}`");
const INVALID_VALUE: CodedMsg = CodedMsg::new_str("invalid value: `{}`");
const DUPLICATED_VALUE: CodedMsg = CodedMsg::new_str("duplicated value");
const CANNOT_CONVERT_FLOAT: CodedMsg = CodedMsg::new_str("cannot convert to float");
const CANNOT_CONVERT_INTEGER: CodedMsg = CodedMsg::new_str("cannot convert to integer");


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

  pub fn load_file(fi: &File) -> Result<Value, Message> {
    Parser::parse_root(&mut Lexer::new(fi))
  }

  pub fn save_file(&self, fpath: String) -> Result<(), String> {
    fs::write(fpath, format!("{}", RawWriter(self, 0))).map_err(|e| e.to_string())
  }

}


struct RawWriter<'a> (&'a Value, usize);

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
        write!(f, "[\n")?;
        for x in v { write!(f, "{}{},\n", "\t".repeat(idn+1), RawWriter(x, idn+1))? }
        write!(f, "{}]", "\t".repeat(idn))?;
      }

      Value::Stc(v) => {
        write!(f, "{{\n")?;
        for (s, x) in v { write!(f, "{}{}: {},\n", "\t".repeat(idn+1), s, RawWriter(x, idn+1))? }
        write!(f, "{}}}", "\t".repeat(idn))?;
      }
      
      Value::Tup(v) => {
        for x in v { write!(f, "{}", RawWriter(x, idn+1))? }
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

  fn parse_root(lex: &mut Lexer) -> Result<Value, Message> {
    match lex.peek()?.kind() {
      WK::Word => Self::parse_struct(lex, true),

      _ => Self::parse_value(lex),
    }
  }

  fn parse_struct(lex: &mut Lexer, is_root: bool) -> Result<Value, Message> {
    let mut ctn = HashMap::new();
    
    if !is_root {
      lex.get()?.expected_kind(WK::CurlyBracketBeg)?;
    }

    loop {
      if is_root {
        if let None = lex.peek_safe() { break }
      } else {
        if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break }
      }

      let name = lex.get()?.expected_word()?;
      let name = lex.str(name).to_string();

      lex.get()?.expected_kind(WK::Colon)?;

      let it = Self::parse_value(lex)?;

      ctn.insert(name, it);

      if is_root { if let None = lex.peek_safe() { break } }

      if let Some(w) = lex.peek_safe() {
        if w.kind() == WK::Comma { lex.bump()? }
      }
    }

    Ok(Value::Stc(ctn))
  }

  fn parse_array(lex: &mut Lexer) -> Result<Value, Message> {
    let mut ctn = vec![];
    lex.get()?.expected_kind(WK::SquareBracketBeg)?;


    loop {
      if lex.peek()?.kind() == WK::SquareBracketEnd { lex.bump()?; break }

      let it = Self::parse_value(lex)?;

      ctn.push(it);

      if let Some(w) = lex.peek_safe() {
        if w.kind() == WK::Comma { lex.bump()? }
      }
    }

    Ok(Value::Arr(ctn))
  }

  fn parse_value(lex: &mut Lexer) -> Result<Value, Message> {
    let mut temp_tuple = vec![];
    let mut mask: u8 = 0;

    loop {
      let (c, ma, val) = match lex.peek_k()? {
        (WK::True | WK::False, _) => { let c = lex.get()?; (c, 1, Value::Bool(c.kind() == WK::True))}

        (WK::String, _) => { let c = lex.get()?; (c, 2, Value::Str(Self::unquote(lex.str(c))))}

        (WK::Number | WK::Sub, _) => {
          let c = lex.get()?;
          
          let (c, sig) = if c.kind() == WK::Sub { (lex.get()?.expected_kind(WK::Number)?, false) } else { (c, true) };

          let str = lex.str(c);
          
          let it = match str.contains('.') {
            true  => Value::Float(str.parse::<f64>().map_err(|_| Message::error(CANNOT_CONVERT_FLOAT, Label::new_pos(c)))? * (if sig {1.} else {-1.})),
            false => Value::Int(str.parse::<i64>().map_err(|_| Message::error(CANNOT_CONVERT_INTEGER, Label::new_pos(c)))? * (if sig {1} else {-1})),
          };

          (c, 4, it)
        }
        
        (WK::CurlyBracketBeg, c) => (c, 8, Self::parse_struct(lex, false)?),
        
        (WK::SquareBracketBeg, c) => (c, 16, Self::parse_array(lex)?),

        (_, c) => return Err(Message::error(INVALID_VALUE, Label::new_args(c, "value is here", &[lex.str(c)])))
      };

      temp_tuple.push(val);

      if (mask & ma) != 0 { return Err(Message::error(DUPLICATED_VALUE, Label::new_pos(c))) } else { mask |= ma }
      
      match lex.peek_safe().map(|w| w.kind()) {
        Some(WK::True | WK::False | WK::String | WK::CurlyBracketBeg | WK::SquareBracketBeg | WK::Number) => {}
        _ => break,
      }
    }

    if temp_tuple.len() == 1 {
      Ok(temp_tuple.pop().unwrap())
    } else {
      Ok(Value::Tup(temp_tuple))
    }
  }

}


trait A {
  fn expected_word(self) -> Result<Word, Message>;
  fn expected_kind(self, kind: WK) -> Result<Word, Message>;
}

impl A for Word {

  fn expected_word(self) -> Result<Self, Message> {
    if self.kind() == WK::Word { Ok(self) }
    else { Err(Message::error(EXPECTED_WORD, Label::new_pos(self))) }
  }

  fn expected_kind(self, kind: WK) -> Result<Self, Message> {
    if self.kind() == kind { Ok(self) }
    else {
      Err(Message::error(EXPECTED_KIND, Label::new_args(self, "{},{}", &[
        &format!("{:?}", kind),
        &format!("{:?}", self.kind()),
      ])))
    }
  }

}
