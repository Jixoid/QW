/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, IdentSave};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{ctx, ExprParser, parse::Ctx};



// Route
pub fn pre_ret(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let label = if lex.peek()?.kind() == WK::Backtick {
    lex.bump()?;
    let name = match lex.get_k()? {
      (WK::Word, c) => c.ident(sin, far)?,
      (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
    };
    Some(name.into())
  } else {
    None
  };

  let val = if lex.peek()?.kind() != WK::Semicolon {
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Return{ label, val }
  };

  Ok(cre.push(this))
}

pub fn pre_break(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let label = if lex.peek()?.kind() == WK::Backtick {
    lex.bump()?;
    let name = match lex.get_k()? {
      (WK::Word, c) => c.ident(sin, far)?,
      (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
    };
    Some(name.into())
  } else {
    None
  };

  let val = if lex.peek()?.kind() != WK::Semicolon {
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Break{ label, val }
  };

  Ok(cre.push(this))
}

pub fn pre_continue(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let label = if lex.peek()?.kind() == WK::Backtick {
    lex.bump()?;
    let name = match lex.get_k()? {
      (WK::Word, c) => c.ident(sin, far)?,
      (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
    };
    Some(name.into())
  } else {
    None
  };

  
  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Continue{ label }
  };

  Ok(cre.push(this))
}
