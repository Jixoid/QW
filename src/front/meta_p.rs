use crate::lexer::WK;
use crate::{ast::*, diagnostic::Message, front::{ParserContext, type_p::TypeParser}, lexer::{Word, WordKind}};


pub struct MetaParser {}

impl MetaParser {

  pub fn pmr_global<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<(),  Message<'a>> {
    let mut level: isize = 0;

    loop {
      let t = ctx.lex.get()?;

      if t.kind == WK::CurlyBracketBeg { level += 1; continue; }
      if t.kind == WK::CurlyBracketEnd {
        level -= 1;

        if level <= 0 { return Ok(()); }
      }
    }
  }


  pub fn read_visibility<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, defvis: &mut Visibility) -> Result<Visibility, Message<'a>> {
    loop {
      let t = ctx.lex.get()?;
      
      let vis = match t.kind {
        WK::Pub   => Visibility::Public,
        WK::Priv  => Visibility::Private,
        WK::Prot  => Visibility::Protected,
        WK::Crate => Visibility::Crate,
        
        _ => {
          ctx.lex.store(t);
          return Ok(defvis.clone());
        },
      };

      let c = ctx.lex.get()?;
      if c.kind == WK::Colon {
        *defvis = vis;
        continue;
      } else {
        ctx.lex.store(c);
        return Ok(vis);
      }
    }
  }
  
  pub fn read_fun_args<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Vec<FieldType<'a>>, Message<'a>> {
    let mut args = vec![];
    
    ctx.lex.get()?.expect_equal_str("(")?;

    'ml: loop {
      let t = ctx.lex.get()?;
      
      if t.kind == WK::ParenEnd { break 'ml; } else { ctx.lex.store(t); }
      
      let mut names: Vec<Word> = vec![];

      're: loop {

        let name = ctx.lex.get()?;
        names.push(name);

        let c = ctx.lex.get()?;

        if c.kind == WK::Comma { continue 're; }
        else if c.kind == WK::Colon { break 're; }
        else {
          c.expect_equal_str2(":", ",")?;
        }
      }

      let kind = TypeParser::read_type(ctx, true)?;
      
      for x in names { args.push(FieldType{name: x, kind, vis: Visibility::Private, attrs: vec![]}); }

      let e = ctx.lex.get()?;

      if e.kind == WK::Comma { continue 'ml; }
      else if e.kind == WK::ParenEnd { break 'ml; }
      else {
        e.expect_equal_str2(",", ")")?;
      }
    }

    Ok(args)
  }

}


impl<'a> Word<'a> {

  pub fn expect_word(self) -> Result<Self, Message<'a>> {
    if self.kind == WordKind::Word {
      Ok(self)
    } else {
      Err(Message::error(self, String::from("expected identifier, but found `{}`"), vec![self.string()]))
    }
  }
  
  pub fn expect_kind(self, kind: WordKind) -> Result<Self, Message<'a>> {
    if self.kind == kind {
      Ok(self)
    } else {
      Err(Message::error(self, String::from("expected {}, but found {}"), vec![format!("{:?}", kind), format!("{:?}", self.kind)]))
    }
  }


  pub fn expect_equal_str(self, s: &str) -> Result<Self, Message<'a>> {
    if self.str() == s {
      Ok(self)
    } else {
      Err(Message::error(self, String::from("expected `{}`, but found `{}`"), vec![s.to_string(), self.string()]))
    }
  }

  pub fn expect_equal_str2(self, s1: &str, s2: &str) -> Result<Self, Message<'a>> {
    if self.str() == s1 || self.str() == s2 {
      Ok(self)
    } else {
      Err(Message::error(self, String::from("expected `{}` or `{}`, but found `{}`"), vec![s1.to_string(), s2.to_string(), self.string()]))
    }
  }
  
  pub fn expect_equal_str3(self, s1: &str, s2: &str, s3: &str) -> Result<Self, Message<'a>> {
    if self.str() == s1 || self.str() == s2 || self.str() == s3 {
      Ok(self)
    } else {
      Err(Message::error(self, String::from("expected `{}`, `{}` or `{}`, but found `{}`"), vec![s1.to_string(), s2.to_string(), s3.to_string(), self.string()]))
    }
  }

}
