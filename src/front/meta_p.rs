use crate::lexer::WK;
use crate::route::build::FileArena;
use crate::{ast::*, diagnostic::Message, front::{ParserContext, type_p::TypeParser}, lexer::{Word, WordKind}};


pub struct MetaParser {}

impl MetaParser {

  pub fn pmr_global(ctx: &mut ParserContext) -> Result<(),  Message> {
    let mut level: isize = 0;

    loop {
      let t = match ctx.lex.get() {
        Ok(t) => t,
        Err(..) => return Ok(()),
      };

      if t.kind == WK::CurlyBracketBeg { level += 1; continue; }
      if t.kind == WK::CurlyBracketEnd {
        level -= 1;

        if level <= 0 { return Ok(()); }
      }
    }
  }


  pub fn read_scpvis(ctx: &mut ParserContext, defvis: &mut Visibility) -> Result<(), Message> {
    loop {
      let t = ctx.lex.get()?;
      
      let vis = match t.kind {
        WK::Pub   => Visibility::Public,
        WK::Priv  => Visibility::Private,
        WK::Prot  => Visibility::Protected,
        WK::Crate => Visibility::Crate,
        WK::Super => Visibility::Super,
        
        _ => {
          ctx.lex.store(t);
          return Ok(());
        }
      };

      let c = ctx.lex.get()?;
      if c.kind == WK::Colon {
        *defvis = vis;
        continue;
      } else {
        ctx.lex.store(c);
        ctx.lex.store(t);
        return Ok(());
      }
    }
  }

  pub fn read_vis(ctx: &mut ParserContext, defvis: Visibility) -> Result<Visibility, Message> {
    let t = ctx.lex.get()?;
    
    let vis = match t.kind {
      WK::Pub   => Visibility::Public,
      WK::Priv  => Visibility::Private,
      WK::Prot  => Visibility::Protected,
      WK::Crate => Visibility::Crate,
      WK::Super => Visibility::Super,
      
      _ => { ctx.lex.store(t); defvis }
    };

    Ok(vis)
  }
  
  pub fn read_fun_args(ctx: &mut ParserContext) -> Result<Rng, Message> {
    let mut args = vec![];
    
    ctx.lex.get()?.expect_kind(WK::ParenBeg)?;

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
          c.expect_kind2(WK::Colon, WK::Comma)?;
        }
      }

      let kind = TypeParser::read_type(ctx, true)?;
      
      for x in names {
        let thing = Thing::NamedType(x.save(ctx), kind);

        args.push( ctx.cre.new_thing(thing).to_any() );
      }

      let e = ctx.lex.get()?;

      if e.kind == WK::Comma { continue 'ml; }
      else if e.kind == WK::ParenEnd { break 'ml; }
      else {
        e.expect_kind2(WK::Comma, WK::ParenBeg)?;
      }
    }

    Ok(ctx.cre.new_extra(args))
  }

}


impl<'a> Word {

  pub fn expect_word(self, far: &FileArena) -> Result<Self, Message> {
    match self.kind == WordKind::Word {
      true  => Ok(self),
      false => Err(Message::error(self, "expected identifier, but found `{}`", vec![
        self.string(far),
      ]))
    }
  }
  

  pub fn expect_kind(self, k1: WK) -> Result<Self, Message> {
    match self.kind == k1 {
      true  => Ok(self),
      false => Err(Message::error(self, "expected {}, but found {}", vec![
        format!("{:?}", k1),
        format!("{:?}", self.kind),
      ]))
    }
  }

  pub fn expect_kind2(self, k1: WK, k2: WK) -> Result<Self, Message> {
    match self.kind == k1 || self.kind == k2 {
      true  => Ok(self),
      false => Err(Message::error(self, "expected `{}` or `{}`, but found `{}`", vec![
        format!("{:?}", k1),
        format!("{:?}", k2),
        format!("{:?}", self.kind),
      ]))
    }
  }
  
  pub fn expect_kind3(self, k1: WK, k2: WK, k3: WK) -> Result<Self, Message> {
    match self.kind == k1 || self.kind == k2 || self.kind == k3 {
      true  => Ok(self),
      false => Err(Message::error(self, "expected `{}`, `{}` or `{}`, but found `{}`", vec![
        format!("{:?}", k1),
        format!("{:?}", k2),
        format!("{:?}", k3),
        format!("{:?}", self.kind),
      ]))
    }
  }

}
