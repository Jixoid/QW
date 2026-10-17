/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, IdentSave, PushOkApi};
use qwc_diagnostic::{ret_err, Label, Message, Status, msg::*};
use qwc_lexer::WK;

use crate::{ctx, ExprParser, parse::Ctx};



// Route
pub fn pre_ret(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let label = if lex.peek(sum)?.kind() == WK::Backtick {
    lex.bump(sum)?;
    let name = match lex.get_k(sum)? {
      (WK::Word, c) => c.ident(sum, sin, far)?,
      (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
    };
    Some(name.into())
  } else {
    None
  };

  let val = if lex.peek(sum)?.kind() != WK::Semicolon {
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Return{ label, val }
  }.push_ok(cre)
}

pub fn pre_break(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let label = if lex.peek(sum)?.kind() == WK::Backtick {
    lex.bump(sum)?;
    let name = match lex.get_k(sum)? {
      (WK::Word, c) => c.ident(sum, sin, far)?,
      (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
    };
    Some(name.into())
  } else {
    None
  };

  let val = if lex.peek(sum)?.kind() != WK::Semicolon {
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };


  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Break{ label, val }
  }.push_ok(cre)
}

pub fn pre_continue(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let label = if lex.peek(sum)?.kind() == WK::Backtick {
    lex.bump(sum)?;
    let name = match lex.get_k(sum)? {
      (WK::Word, c) => c.ident(sum, sin, far)?,
      (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
    };
    Some(name.into())
  } else {
    None
  };

  
  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Continue{ label }
  }.push_ok(cre)
}
