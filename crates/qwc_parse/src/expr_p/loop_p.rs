use qwc_ast::{Expr, ExprId, ExprKind};
use qwc_diagnostic::Message;
use qwc_lexer::WK;

use crate::{ExprParser, PattParser, ctx, WordCheck, parse::Ctx};


// Loop
pub fn pre_loop(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = if lex.peek()?.kind() == WK::Else {
    lex.bump()?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };

  
  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Loop{ blok, elsb }
  };

  Ok(cre.push(this))
}

pub fn pre_while(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let cond = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
  
  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = if lex.peek()?.kind() == WK::Else {
    lex.bump()?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::While{ cond, blok, elsb }
  };

  Ok(cre.push(this))
}

pub fn pre_for(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let vars = PattParser::read_patt(ctx!(cre, sin, far, lex, sum, side))?;

  lex.get()?.expect_kind(WK::In)?;
  
  let iter = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = if lex.peek()?.kind() == WK::Else {
    lex.bump()?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::ForIn{ vars, iter, blok, elsb }
  };

  Ok(cre.push(this))
}
