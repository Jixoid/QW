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

use crate::{ExprParser, ctx, WordCheck, parse::Ctx, PattParser, TypeParser};


// Block
impl ExprParser {

  pub fn pre_block(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let label = match start.kind() {
      WK::BraceL => None,

      WK::Backtick => {
        let name = match lex.get_k(sum)? {
          (WK::Word, c) => c.ident(sum, sin, far)?,
          (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
        };
        
        lex.get(sum)?.expect_kind(sum, WK::Colon)?;
        lex.get(sum)?.expect_kind(sum, WK::BraceL)?;
        
        Some(name.into())
      }
      
      _ => start.panic_kind2(sum, WK::Backtick, WK::BraceL)?
    };

    let (rng, expr) = {
      let mut ctn = vec![];
      let mut expr = None;
      
      loop {
        match lex.peek(sum)?.kind() {
          WK::BraceR => { lex.bump(sum)?; break }
          WK::Semicolon => { lex.bump(sum)?; continue; }
          _ => {}
        }
        
        let ex_id = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
        
        match lex.peek_k(sum)? {
          (WK::Semicolon, _) => { lex.bump(sum)?; ctn.push(ex_id); },
          
          (WK::BraceR, _) => { expr = Some(ex_id); }
          
          (_, c) => {
            if cre.get(ex_id).is_like_blok() {
              ctn.push(ex_id);
            } else {
              c.panic_kind(sum, WK::Semicolon)?;
            }
          }
        }
      }

      (cre.extra(&ctn), expr)
    };


    // Post
    Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Block{ label, rng, expr }
    }.push_ok(cre)
  }

}


// Context
pub fn pre_unsafe(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;


  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Unsafe(blok)
  }.push_ok(cre)
}

pub fn pre_relaxed(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;


  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Relaxed(blok)
  }.push_ok(cre)
}


// Variable
pub fn pre_let(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  let item = PattParser::read_patt(ctx!(cre, sin, far, lex, sum, side))?;

  let kind = if lex.peek(sum)?.kind() == WK::Colon {
    lex.bump(sum)?;
    Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };

  let init = if lex.peek(sum)?.kind() == WK::Eq {
    lex.bump(sum)?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  }
  else {
    None
  };


  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Let{ item, kind, init, ism: start.kind() == WK::Var }
  }.push_ok(cre)
}
