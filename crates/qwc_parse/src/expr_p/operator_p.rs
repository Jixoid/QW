/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, PushOkApi, Rng};
use qwc_diagnostic::{Span, Status};
use qwc_lexer::WK;

use super::helper;
use crate::{ExprParser, ctx, meta_p::WordCheck, parse::Ctx};



// Unary
pub fn pre_unary(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let op = helper::parse_unary_op(lex, sum, start.kind())?;
  
  let val = ExprParser::read_expr_sub(ctx!(cre, sin, far, lex, sum, side), 85)?;
  

  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Unary{ op, val }
  }.push_ok(cre)
}

pub fn post_unary(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let op = lex.get(sum)?;

  let op = helper::parse_unary_op(lex, sum, op.kind())?;
  

  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Unary{ op, val: lhs }
  }.push_ok(cre)
}


// Call & Index
pub fn post_call(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get(sum)?;

  let args = if lex.peek(sum)?.kind() == WK::ParenR {
    lex.bump(sum)?;
    Rng::empty()
  } else {
    let mut args = vec![];

    loop {
      args.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);

      match lex.get_k(sum)? {
        (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break },
        
        (WK::ParenR, _) => break,

        (_, c) => c.panic_kind2(sum, WK::Comma, WK::ParenR)?,
      }
    }

    cre.extra(&args)
  };

  
  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Call{ callee: lhs, args }
  }.push_ok(cre)
}

pub fn post_index(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get(sum)?;

  let args = if lex.peek(sum)?.kind() == WK::BracketR {
    lex.bump(sum)?;
    Rng::empty()
  } else {
    let mut args = vec![];

    loop {
      args.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);

      match lex.get_k(sum)? {
        (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::BracketR { lex.bump(sum)?; break },
        
        (WK::BracketR, _) => break,

        (_, c) => c.panic_kind2(sum, WK::Comma, WK::BracketR)?,
      }
    }

    cre.extra(&args)
  };

  
  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Index{ callee: lhs, args }
  }.push_ok(cre)
}
