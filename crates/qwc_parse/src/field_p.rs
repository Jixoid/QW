use qwc_ast::{Field, FieldId, FieldKind, IdentSave, Rng, Thing, Visibility};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{ctx, ExprParser, MetaParser, WordCheck, Ctx, TypeParser};


pub struct FieldParser;

impl FieldParser {

  // Public
  pub fn read_field(ctx: &mut Ctx, defvis: &mut Visibility) -> Result<FieldId, Message> {
    let (vis, attr) = MetaParser::read_start(ctx, defvis)?;

    let id = match ctx.lex.peek_k()? {
      (WK::Word, _)    => Self::sub_member (ctx, vis)?,
      (WK::Type, _)    => Self::sub_type   (ctx, vis)?,
      (WK::Fun, _)     => Self::sub_fun    (ctx, vis)?,
      (WK::Init, _)    => Self::sub_init   (ctx, vis)?,
      (WK::Fini, _)    => Self::sub_fini   (ctx, vis)?,
      (WK::Impl, _)    => Self::sub_impl   (ctx, vis)?,


      (_, c) => return Err(Message::error(UNKNOWN_KEYWORD, Label::new_pos(c))),
    };

    if let Some(a) = attr { ctx.cre.attach(id, a); }
    
    Ok(id)
  }


  // Sub
  fn sub_member(ctx: &mut Ctx, vis: Visibility) -> Result<FieldId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    let name = lex.get()?.ident(sin, far)?;

    lex.get()?.expect_kind(WK::Colon)?;

    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::Semicolon)?;


    // Post
    let this = Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::MemberVar {kind}
    };
    
    Ok(cre.push(this))
  }


  fn sub_type(ctx: &mut Ctx, vis: Visibility) -> Result<FieldId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;
    
    let kind = match lex.peek_k()? {
      (WK::Eq, _) => {
        lex.get()?.expect_kind(WK::Eq)?;
        let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;
        lex.get()?.expect_kind(WK::Semicolon)?;
        Some(kind)
      }
      
      (WK::Semicolon, _) => {
        lex.get()?.expect_kind(WK::Semicolon)?;
        None
      }

      (_, c) => c.panic_kind2(WK::Eq, WK::Semicolon)?,
    };

    
    // Post
    let this = Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Type(kind),
    }; 

    Ok(cre.push(this))
  }


  fn sub_fun(ctx: &mut Ctx, vis: Visibility) -> Result<FieldId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum))?;

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Fun{ kind, blok },
    };
    
    Ok(cre.push(this))
  }

  fn sub_init(ctx: &mut Ctx, vis: Visibility) -> Result<FieldId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum))?;

    let ils = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      let mut ils = vec![];
      
      loop {
        let name = if let (WK::Word, c) = lex.peek_k()? { lex.bump()?; c.ident(sin, far)? } else { break };
        
        lex.get()?.expect_kind(WK::ParenL)?;
        
        let v = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
        lex.get()?.expect_kind(WK::ParenR)?;
        
        let it = Thing::NamedExpr(name, v);
        ils.push(cre.push(it));
        
        if lex.peek()?.kind() == WK::Comma { lex.bump()? } else { break }
      }
      
      cre.extra(&ils)
    } else {
      Rng::empty()
    };

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Init{ kind, blok, ils }
    };

    Ok(cre.push(this))
  }
  
  fn sub_fini(ctx: &mut Ctx, vis: Visibility) -> Result<FieldId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum))?;

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Fini{ kind, blok }
    };

    Ok(cre.push(this))
  }


  fn sub_impl(ctx: &mut Ctx, vis: Visibility) -> Result<FieldId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    lex.get()?.expect_kind(WK::Colon)?;

    let trait_ty = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::BraceL)?;

    let ctn = {
      let mut impls = vec![];
      let defvis = &mut Visibility::Inherited;
      
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let (vis, attrs) = MetaParser::read_start(ctx!(cre, sin, far, lex, sum), defvis)?;
        
        let next_kw = lex.peek_k()?;
        if next_kw.0 == WK::Fun {
          let fun = Self::sub_fun(ctx!(cre, sin, far, lex, sum), vis)?;
          
          if let Some(a) = attrs { cre.attach(fun, a); }
          
          impls.push(fun);
        } else {
          let c = lex.get()?;
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(c)));
        }
      }
      
      cre.extra(&impls)
    };


    // Post
    let this = Field {
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: FieldKind::ImplIn{ trait_ty, ctn },
    };
    
    Ok(cre.push(this))
  }
  

}
