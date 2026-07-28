use std::fs;
use crate::{control::module::{self, CompilerError, ModuleFile}, lexer::{Lexer, Word, WordKind}};

#[derive(Clone, Debug, PartialEq)]
pub struct TypeArr {
  pub subs: Vec<Value>,
}

impl TypeArr {
  pub fn new() -> TypeArr { return TypeArr{subs: Vec::new()}; }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubTypeStc {
  pub kind: Value,
  pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeStc {
  pub subs: Vec<SubTypeStc>,
}

impl TypeStc {
  pub fn new() -> TypeStc { return TypeStc{subs: Vec::new()}; }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
  Null,
  I64(i64),
  F64(f64),
  Boo(bool),
  Str(String),
  Typ(String, String),
  Ref(String),
  Arr(TypeArr),
  Stc(TypeStc),
  Tup(TypeArr),
}

impl Value {
  pub fn make_null() -> Value { return Value::Null; }
  pub fn make_int(v: i64) -> Value { return Value::I64(v); }
  pub fn make_float(v: f64) -> Value { return Value::F64(v); }
  pub fn make_bool(v: bool) -> Value { return Value::Boo(v); }
  pub fn make_string(v: String) -> Value { return Value::Str(v); }
  pub fn make_str(v: &str) -> Value { return Value::Str(String::from(v)); }
  pub fn make_type(a: String, b: String) -> Value { return Value::Typ(a, b); }
  pub fn make_ref(v: String) -> Value { return Value::Ref(v); }
  pub fn make_struct() -> Value { return Value::Stc(TypeStc::new()); }
  pub fn make_array() -> Value { return Value::Arr(TypeArr::new()); }
  pub fn make_tuple() -> Value { return Value::Tup(TypeArr::new()); }

  pub fn push_array(&mut self, v: Value) -> module::Result<()> {
    match self {
      Self::Arr(subs) => subs.subs.push(v),
      _ => return Err(CompilerError::Ds(String::from("type is not array"))),
    };

    Ok(())
  }

  pub fn push_struct(&mut self, s: &str, v: Value) -> module::Result<()> {
    match self {
      Self::Stc(subs) => subs.subs.push(SubTypeStc {kind: v, name: String::from(s) }),
      _ => return Err(CompilerError::Ds(String::from("type is not struct"))),
    };

    Ok(())
  }


  pub fn load_file(m: &ModuleFile) -> Result<Value, String> {
    let mut lexer = Lexer::new_module(m);
    let mut parser = Parser::new(&mut lexer);
    parser.parse()
  }

  pub fn save_file(&self, fpath: String) -> Result<(), String> {
    let mut out = String::new();
    self.write_raw(&mut out, 0, true);
    fs::write(fpath, out).map_err(|e| e.to_string())
  }

  fn write_raw(&self, out: &mut String, mut indent: usize, is_root: bool) {
    let write_indent = |out: &mut String, ind: usize| {
      for _ in 0..ind {
        out.push_str("  ");
      }
    };

    let write_escaped_string = |out: &mut String, s: &str| {
      out.push('"');
      for c in s.chars() {
        match c {
          '"' => out.push_str("\\\""),
          '\\' => out.push_str("\\\\"),
          '\n' => out.push_str("\\n"),
          '\t' => out.push_str("\\t"),
          '\r' => out.push_str("\\r"),
          _ => out.push(c),
        }
      }
      out.push('"');
    };

    match self {
      Value::Null => out.push_str("null"),
      Value::I64(v) => out.push_str(&v.to_string()),
      Value::F64(v) => out.push_str(&v.to_string()),
      Value::Boo(v) => out.push_str(if *v { "true" } else { "false" }),
      Value::Str(v) => write_escaped_string(out, v),
      Value::Typ(a, b) => {
        out.push_str(a);
        out.push_str("::");
        out.push_str(b);
      }
      Value::Ref(v) => {
        out.push('@');
        out.push_str(v);
      }
      Value::Arr(arr) | Value::Tup(arr) => {
        out.push('[');
        let sz = arr.subs.len();
        for (i, v) in arr.subs.iter().enumerate() {
          v.write_raw(out, indent, false);
          if i + 1 < sz {
            out.push(',');
          }
        }
        out.push(']');
      }
      Value::Stc(stc) => {
        if !is_root {
          out.push_str("{\n");
          indent += 1;
        }
        for sub in &stc.subs {
          if !is_root {
            write_indent(out, indent);
          }
          out.push_str(&sub.name);
          out.push_str(": ");
          sub.kind.write_raw(out, indent, false);
          out.push('\n');
        }
        if !is_root {
          indent -= 1;
          write_indent(out, indent);
          out.push('}');
        }
      }
    }
  }
}


struct Parser<'a, 'c> {
  lexer: &'c mut Lexer<'a>,
  tokens: Vec<Word<'a>>,
  pos: usize,
}

impl<'a, 'c> Parser<'a, 'c> {

  fn new(lexer: &'c mut Lexer<'a>) -> Self {
    let mut tokens = Vec::new();
    loop {
      let w = lexer.lex();
      if w.kind == WordKind::EOF {
        break;
      }
      if w.kind != WordKind::Unknown {
        tokens.push(w);
      }
    }
    Self {lexer, tokens, pos: 0}
  }

  fn peek(&self, offset: usize) -> Option<Word<'a>> {
    self.tokens.get(self.pos + offset).copied()
  }

  fn next(&mut self) -> Option<Word<'a>> {
    let w = self.tokens.get(self.pos).copied();
    if w.is_some() {
      self.pos += 1;
    }
    w
  }

  fn text(&self, w: &Word<'a>) -> String {
    self.lexer.mol.mmap[w.off .. w.off + w.size].to_string()
  }

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

  fn parse(&mut self) -> Result<Value, String> {
    let mut stc = TypeStc::new();
    self.parse_struct_content(&mut stc)?;
    Ok(Value::Stc(stc))
  }

  fn parse_struct_content(&mut self, stc: &mut TypeStc) -> Result<(), String> {
    while let Some(w) = self.peek(0) {
      if w.kind == WordKind::CurlyBracketEnd {
        self.next(); // consume }
        return Ok(());
      }

      let name_word = self.next().unwrap().clone();
      if name_word.kind != WordKind::Word {
        return Err(format!("invalid identifier: {}", self.text(&name_word)));
      }
      let name = self.text(&name_word);

      if let Some(c) = self.peek(0) {
        if c.kind == WordKind::Colon {
          self.next(); // consume :
        }
      }

      let val = self.parse_value(true)?;
      stc.subs.push(SubTypeStc { kind: val, name });

      if let Some(c) = self.peek(0) {
        if c.kind == WordKind::Comma {
          self.next(); // consume ,
        }
      }
    }
    Ok(())
  }

  fn parse_array_content(&mut self, arr: &mut TypeArr) -> Result<(), String> {
    while let Some(w) = self.peek(0) {
      if w.kind == WordKind::SquareBracketEnd {
        self.next(); // consume ]
        return Ok(());
      }

      let val = self.parse_value(false)?;
      arr.subs.push(val);

      if let Some(c) = self.peek(0) {
        if c.kind == WordKind::Comma {
          self.next(); // consume ,
        }
      }
    }
    Ok(())
  }

  fn parse_value(&mut self, is_struct_context: bool) -> Result<Value, String> {
    let mut temp_tuple = Vec::new();
    let mut mask: u8 = 0;

    loop {
      let val: Value;
      let mut is_node = false;

      let w = match self.next() {
        Some(word) => word.clone(),
        None => return Err("Unexpected EOF while parsing value".to_string()),
      };
      let word_text = self.text(&w);

      if w.kind == WordKind::Word && (word_text == "true" || word_text == "false" || word_text == "yes" || word_text == "no") {
        if mask & 1 != 0 { return Err("tuple: duplicate bool".to_string()); }
        mask |= 1;
        val = Value::make_bool(word_text == "true" || word_text == "yes");
      } else if w.kind == WordKind::String {
        if mask & 2 != 0 { return Err("tuple: duplicate string".to_string()); }
        mask |= 2;
        val = Value::make_string(Self::unquote(&word_text));
      } else if w.kind == WordKind::Word && self.peek(0).map(|x| x.kind) == Some(WordKind::Scope) {
        if mask & 4 != 0 { return Err("tuple: duplicate type".to_string()); }
        mask |= 4;
        self.next(); // consume ::
        let t2 = self.next().ok_or("expected text after ::")?.clone();
        if t2.kind != WordKind::Word { return Err("expected text after ::".to_string()); }
        val = Value::make_type(word_text, self.text(&t2));
      } else if w.kind == WordKind::At {
        if mask & 8 != 0 { return Err("tuple: duplicate reference".to_string()); }
        mask |= 8;
        let t2 = self.next().ok_or("expected string or ^ after @")?.clone();
        let t2_text = self.text(&t2);
        if t2.kind != WordKind::String && t2_text != "^" { return Err("expected string or ^ after @".to_string()); }
        val = Value::make_ref(Self::unquote(&t2_text));
      } else if w.kind == WordKind::Word && (word_text.chars().next().unwrap().is_ascii_digit() || word_text.starts_with("-")) {
        if mask & 16 != 0 { return Err("tuple: duplicate number".to_string()); }
        mask |= 16;
        
        let mut num_str = word_text;
        let mut is_float = false;
        
        if self.peek(0).map(|x| x.kind) == Some(WordKind::Dot) {
          self.next(); // consume dot
          let t2 = self.next().ok_or("expected fractional part")?.clone();
          is_float = true;
          num_str = format!("{}.{}", num_str, self.text(&t2));
        }
        
        if is_float {
          let f = num_str.parse::<f64>().map_err(|_| "convert to float failed")?;
          val = Value::make_float(f);
        } else {
          let i = num_str.parse::<i64>().map_err(|_| "convert to integer failed")?;
          val = Value::make_int(i);
        }
      } else if w.kind == WordKind::Sub {
        if mask & 16 != 0 { return Err("tuple: duplicate number".to_string()); }
        mask |= 16;
        
        let t2 = self.next().ok_or("expected number after -")?.clone();
        let mut num_str = format!("-{}", self.text(&t2));
        let mut is_float = false;
        
        if self.peek(0).map(|x| x.kind) == Some(WordKind::Dot) {
          self.next(); // consume dot
          let t3 = self.next().ok_or("expected fractional part")?.clone();
          is_float = true;
          num_str = format!("{}.{}", num_str, self.text(&t3));
        }
        
        if is_float {
          let f = num_str.parse::<f64>().map_err(|_| "convert to float failed")?;
          val = Value::make_float(f);
        } else {
          let i = num_str.parse::<i64>().map_err(|_| "convert to integer failed")?;
          val = Value::make_int(i);
        }
      } else if w.kind == WordKind::CurlyBracketBeg {
        if mask & 32 != 0 { return Err("tuple: duplicate struct".to_string()); }
        mask |= 32;
        let mut stc = TypeStc::new();
        self.parse_struct_content(&mut stc)?;
        val = Value::Stc(stc);
        is_node = true;
      } else if w.kind == WordKind::SquareBracketBeg {
        if mask & 64 != 0 { return Err("tuple: duplicate array".to_string()); }
        mask |= 64;
        let mut arr = TypeArr::new();
        self.parse_array_content(&mut arr)?;
        val = Value::Arr(arr);
        is_node = true;
      } else if w.kind == WordKind::Word && word_text == "null" {
        if mask & 128 != 0 { return Err("tuple: duplicate null".to_string()); }
        mask |= 128;
        val = Value::make_null();
      } else {
        return Err(format!("invalid value: {}", word_text));
      }

      temp_tuple.push(val);

      let mut should_continue = false;
      if !is_node {
        if let Some(nx) = self.peek(0) {
          if nx.kind != WordKind::Comma && nx.kind != WordKind::CurlyBracketEnd && nx.kind != WordKind::SquareBracketEnd {
            let mut is_next_key = false;
            if is_struct_context && nx.kind == WordKind::Word {
              let nx_text = self.text(&nx);
              if nx_text == "true" || nx_text == "false" || nx_text == "yes" || nx_text == "no" {
                is_next_key = false;
              } else if self.peek(1).map(|x| x.kind) == Some(WordKind::Scope) {
                is_next_key = false;
              } else {
                is_next_key = true;
              }
            }
            if !is_next_key {
              should_continue = true;
            }
          }
        }
      }

      if !should_continue {
        break;
      }
    }

    if temp_tuple.len() == 1 {
      Ok(temp_tuple.pop().unwrap())
    } else {
      let mut tup_arr = TypeArr::new();
      tup_arr.subs = temp_tuple;
      Ok(Value::Tup(tup_arr))
    }
  }

}
