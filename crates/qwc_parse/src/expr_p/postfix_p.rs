use qwc_ast::{AnyRng, Expr, ExprId, ExprKind};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{ctx, expr_p::{ExprParser, literal_p}, meta_p::WordCheck, parse::Ctx, type_p::TypeParser};


// Postfix
pub fn post_scope_spec(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  lex.get()?;

  match lex.peek()?.kind() {
    WK::Lt => post_specialize(ctx!(cre, sin, far, lex, sum), lhs),
    _ => post_scope(ctx!(cre, sin, far, lex, sum), lhs),
  }
}


pub fn post_member(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  let start = lex.get()?;

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

pub fn post_scope(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  let start = lex.peek()?;

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

  Ok(cre.push(this))
}


pub fn post_specialize(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
  let start = lex.get()?;

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

  Ok(cre.push(this))
}
