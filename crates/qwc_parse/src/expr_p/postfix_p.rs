use qwc_ast::{AnyRng, Expr, ExprId, ExprKind, IdentSave, Thing};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_lexer::WK;

use crate::{ctx, expr_p::{ExprParser, literal_p, postfix_p}, meta_p::WordCheck, parse::Ctx, type_p::TypeParser};


// Postfix
pub fn post_scope_spec(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> {
  ctx.lex.get()?;

  match ctx.lex.peek()?.kind() {
    WK::Lt => post_specialize(ctx, start, lhs),
    _ => post_scope(ctx, start, lhs),
  }
}


pub fn post_member(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  lex.get()?;

  let rng = {
    let sub = literal_p::pre_nick(ctx!(cre, sin, far, lex, sum))?;
    
    let mut ctn = vec![lhs, sub];

    loop {
      match lex.peek_k()? {
        (WK::Dot, _) => {
          lex.bump()?;
          ctn.push(literal_p::pre_nick(ctx!(cre, sin, far, lex, sum))?);
        }

        (WK::Colon2, c) => return Err(Message::error(CANNOT_FIELD_ACCESS_AFTER_MEMBER, Label::new_pos(c))),

        _ => break
      }
    }
    
    cre.extra(&ctn)
  };
    

  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Member(rng)
  };

  Ok(cre.push(this))
}

pub fn post_scope(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  lex.peek()?;

  let rng = {
    let sub = literal_p::pre_nick(ctx!(cre, sin, far, lex, sum))?;
    
    let mut ctn = vec![lhs, sub];

    loop {
      if lex.peek()?.kind() == WK::Colon2 {
        lex.bump()?;
        ctn.push(literal_p::pre_nick(ctx!(cre, sin, far, lex, sum))?);
      } else {
        break
      }
    }
    
    cre.extra(&ctn)
  };
    

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Path(rng)
  };
  let this = cre.push(this);
  
  if lex.peek()?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start.into(), this) } else { Ok(this) }
}


pub fn post_specialize(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  lex.get()?;

  let args = if lex.peek()?.kind() == WK::Gt {
    lex.bump()?;
    AnyRng::empty()
  } else {
    let mut args = vec![];

    loop {
      let arg = match lex.peek_k()? {
        (WK::String | WK::Number | WK::BraceL, _) => ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?.to_any(),
        
        _ => TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?.to_any(),
      };
      
      args.push(arg);

      match lex.get_k()? {
        (WK::Comma, _) => if lex.peek()?.kind() == WK::Gt { lex.bump()?; break },
        
        (WK::Gt, _) => break,

        (_, c) => c.panic_kind2(WK::Comma, WK::Gt)?,
      }
    }

    cre.extra_any(&args)
  };

  
  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Spec{ callee: lhs, args }
  };
  let this = cre.push(this);

  if lex.peek()?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start.into(), this) } else { Ok(this) }
}


// FieldCreate
pub fn post_field_create(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  let s = lex.get()?;
  
  let fields = {
    let mut vec = vec![];
    
    loop {
      if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
      
      let name = lex.get()?.ident(sin, far)?;
      
      lex.get()?.expect_kind(WK::Colon)?;
      
      let expr = ExprParser::read_expr(ctx!(cre,sin,far,lex,sum))?;


      vec.push(cre.push(Thing::NamedExpr(name, expr)));

      match lex.get_k()? {
        (WK::Comma, _)  => continue,
        (WK::BraceR, _) => break,

        (_, c) => c.panic_kind2(WK::Comma, WK::BraceR)?,
      }
    }

    cre.extra(&vec)
  };


  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::FieldCreate { lhs, fields, brace_span: lex.pos_extend(s) }
  };

  Ok(cre.push(this))
}
