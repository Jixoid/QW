/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, PushOkApi, Thing, ThingKind};
use qwc_diagnostic::Status;
use qwc_lexer::WK;

use crate::{ExprParser, WordCheck, ctx, expr_p::ExprRestriction, parse::Ctx};


// Condition
pub fn pre_if(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let cond = ExprParser::rest_read_expr(ctx!(cre, sin, far, lex, sum, side), ExprRestriction::NO_FIELD_INIT)?;
  
  let then = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = match lex.peek_k(sum)? {
    (WK::Ef, _) => Some(pre_if(ctx!(cre, sin, far, lex, sum, side))?),

    (WK::Else, _) => {
      lex.bump(sum)?;
      Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?)
    }
    _ => None,
  };


  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::If{ cond, then, elsb }
  }.push_ok(cre)
}

pub fn pre_match(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let cond = ExprParser::rest_read_expr(ctx!(cre, sin, far, lex, sum, side), ExprRestriction::NO_FIELD_INIT)?;
  
  let arms = {
    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;
    
    let mut arm_ids = vec![];
    
    loop {
      if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }

      let start = lex.peek(sum)?;
      
      let pat = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
      lex.get(sum)?.expect_kind(sum, WK::FatArrow)?;
      let body = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
      
      arm_ids.push(Thing {
        kind: ThingKind::MatchArm(pat, body),
        pos: lex.pos_extend(start),
      }.push(cre));
      
      match lex.peek_k(sum)? {
        (WK::BraceR, _) => { lex.bump(sum)?; break }
        (WK::Comma, _) => { lex.bump(sum)?; }

        (_, c) => c.panic_kind2(sum, WK::BraceR, WK::Comma)?
      }
    }

    cre.extra(&arm_ids)
  };


  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Match{ cond, arms }
  }.push_ok(cre)
}
