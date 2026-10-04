use qwc_ast::{Expr, ExprId, ExprKind, Rng};
use qwc_diagnostic::{Message, Span};
use qwc_lexer::WK;

use super::helper;
use crate::{ExprParser, ctx, meta_p::WordCheck, parse::Ctx};



// Unary
pub fn pre_unary(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let op = helper::parse_unary_op(start.kind());
  
  let val = ExprParser::read_expr_sub(ctx!(cre, sin, far, lex, sum, side), 85)?;
  

  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Unary{ op, val }
  };

  Ok(cre.push(this))
}

pub fn post_unary(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let op = lex.get()?;

  let op = helper::parse_unary_op(op.kind());
  

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Unary{ op, val: lhs }
  };

  Ok(cre.push(this))
}


// Call & Index
pub fn post_call(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get()?;

  let args = if lex.peek()?.kind() == WK::ParenR {
    lex.bump()?;
    Rng::empty()
  } else {
    let mut args = vec![];

    loop {
      args.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);

      match lex.get_k()? {
        (WK::Comma, _) => if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break },
        
        (WK::ParenR, _) => break,

        (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?,
      }
    }

    cre.extra(&args)
  };

  
  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Call{ callee: lhs, args }
  };

  Ok(cre.push(this))
}

pub fn post_index(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get()?;

  let args = if lex.peek()?.kind() == WK::BracketR {
    lex.bump()?;
    Rng::empty()
  } else {
    let mut args = vec![];

    loop {
      args.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);

      match lex.get_k()? {
        (WK::Comma, _) => if lex.peek()?.kind() == WK::BracketR { lex.bump()?; break },
        
        (WK::BracketR, _) => break,

        (_, c) => c.panic_kind2(WK::Comma, WK::BracketR)?,
      }
    }

    cre.extra(&args)
  };

  
  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Index{ callee: lhs, args }
  };

  Ok(cre.push(this))
}
