/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, Thing};
use qwc_diagnostic::Message;
use qwc_lexer::WK;

use crate::{ctx, ExprParser, WordCheck, parse::Ctx};


// Condition
pub fn pre_if(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let cond = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
  
  let then = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;

  let elsb = match lex.peek_k()? {
    (WK::Ef, _) => Some(pre_if(ctx!(cre, sin, far, lex, sum, side))?),

    (WK::Else, _) => {
      lex.bump()?;
      Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?)
    }
    _ => None,
  };


  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::If{ cond, then, elsb }
  };

  Ok(cre.push(this))
}

pub fn pre_match(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let cond = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
  
  let arms = {
    lex.get()?.expect_kind(WK::BraceL)?;
    
    let mut arm_ids = vec![];
    
    loop {
      if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
      
      let pat = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
      lex.get()?.expect_kind(WK::FatArrow)?;
      let body = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
      
      let arm = Thing::MatchArm(pat, body);
      arm_ids.push(cre.push(arm));
      
      match lex.peek_k()? {
        (WK::BraceR, _) => { lex.bump()?; break }
        (WK::Comma, _) => { lex.bump()?; }

        (_, c) => c.panic_kind2(WK::BraceR, WK::Comma)?
      }
    }

    cre.extra(&arm_ids)
  };


  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Match{ cond, arms }
  };

  Ok(cre.push(this))
}
