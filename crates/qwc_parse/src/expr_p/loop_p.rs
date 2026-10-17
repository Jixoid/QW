/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, PushOkApi};
use qwc_diagnostic::Status;
use qwc_lexer::WK;

use crate::{ExprParser, PattParser, WordCheck, ctx, expr_p::ExprRestriction, parse::Ctx};


// Loop
pub fn pre_loop(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = if lex.peek(sum)?.kind() == WK::Else {
    lex.bump(sum)?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };

  
  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Loop{ blok, elsb }
  }.push_ok(cre)
}

pub fn pre_while(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let cond = ExprParser::rest_read_expr(ctx!(cre, sin, far, lex, sum, side), ExprRestriction::NO_FIELD_INIT)?;
  
  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = if lex.peek(sum)?.kind() == WK::Else {
    lex.bump(sum)?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::While{ cond, blok, elsb }
  }.push_ok(cre)
}

pub fn pre_for(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let vars = PattParser::read_patt(ctx!(cre, sin, far, lex, sum, side))?;

  lex.get(sum)?.expect_kind(sum, WK::In)?;
  
  let iter = ExprParser::rest_read_expr(ctx!(cre, sin, far, lex, sum, side), ExprRestriction::NO_FIELD_INIT)?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = if lex.peek(sum)?.kind() == WK::Else {
    lex.bump(sum)?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::ForIn{ vars, iter, blok, elsb }
  }.push_ok(cre)
}
