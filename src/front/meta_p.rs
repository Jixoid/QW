use crate::{ast::*, diagnostic::Message, front::{ParserContext, type_p::TypeParser}, lexer::Word};


pub struct MetaParser {}

impl MetaParser {

  pub fn pmr_global<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<(),  Message<'a>> {
    let mut level: isize = 0;

    loop {
      let t = ctx.lex.get()?;

      if t.str() == "{" { level += 1; continue; }
      if t.str() == "}" {
        level -= 1;

        if level <= 0 { return Ok(()); }
      }
    }
  }


  pub fn expect_equal<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, s: &str) -> Result<(), Message<'a>> {
    let t = ctx.lex.get()?;

    if t.str() == s {
      Ok(())
    } else {
      Err(Message::error(t, String::from("expected {}, but found {}"), vec![s.to_string(), t.string()]))
    }
  }

  pub fn expect_equal_w<'a>(w: Word<'a>, s: &str) -> Result<(), Message<'a>> {
    if w.str() == s {
      Ok(())
    } else {
      Err(Message::error(w, String::from("expected {}, but found {}"), vec![s.to_string(), w.string()]))
    }
  }

  pub fn expect_equal2_w<'a>(w: Word<'a>, s1: &str, s2: &str) -> Result<(), Message<'a>> {
    if w.str() == s1 || w.str() == s2 {
      Ok(())
    } else {
      Err(Message::error(w, String::from("expected {} or {}, but found {}"), vec![s1.to_string(), s2.to_string(), w.string()]))
    }
  }


  pub fn read_visibility<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, defvis: &mut Visibility) -> Result<Visibility, Message<'a>> {
    loop {
      let t = ctx.lex.get()?;
      
      let vis = match t.str() {
        "pub"   => Visibility::Public,
        "priv"  => Visibility::Private,
        "prot"  => Visibility::Protected,
        "crate" => Visibility::Crate,
        _ => {
          ctx.lex.store(t);
          return Ok(defvis.clone());
        },
      };

      let c = ctx.lex.get()?;
      if c.str() == ":" {
        *defvis = vis;
        continue;
      } else {
        ctx.lex.store(c);
        return Ok(vis);
      }
    }
  }
  
  pub fn read_fun_args<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Vec<FieldType<'a>>, Message<'a>> {
    let mut args = Vec::new();
    
    Self::expect_equal(ctx, "(")?;

    'ml: loop {
      let t = ctx.lex.get()?;
      
      if t.str() == ")" { break 'ml; } else { ctx.lex.store(t); }
      
      let mut names: Vec<Word> = vec![];

      're: loop {

        let name = ctx.lex.get()?;
        names.push(name);


        let c = ctx.lex.get()?;

        if c.str() == "," { continue 're; }
        else if c.str() == ":" { break 're; }
        else {
          Self::expect_equal2_w(c, ":", ",")?;
        }
      }


      let kind = TypeParser::read_type(ctx, true)?;


      for x in names { args.push(FieldType{
        name: x, kind: kind, vis: Visibility::Public
      })}
      

      let e = ctx.lex.get()?;

      if e.str() == ";" { continue 'ml; }
      else if e.str() == ")" { break 'ml; }
      else {
        Self::expect_equal2_w(e, ";", ")")?;
      }
    }

    Ok(args)
  }

}
