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

use crate::{ExprParser, ctx, WordCheck, parse::Ctx, PattParser, TypeParser};


// Block
impl ExprParser {

  pub fn pre_block(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let label = match start.kind() {
      WK::BraceL => None,

      WK::Backtick => {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
        };
        
        lex.get()?.expect_kind(WK::Colon)?;
        lex.get()?.expect_kind(WK::BraceL)?;
        
        Some(name.into())
      }
      
      _ => start.panic_kind2(WK::Backtick, WK::BraceL)?
    };

    let (rng, expr) = {
      let mut ctn = vec![];
      let mut expr = None;
      
      loop {
        match lex.peek()?.kind() {
          WK::BraceR => { lex.bump()?; break }
          WK::Semicolon => { lex.bump()?; continue; }
          _ => {}
        }
        
        let ex_id = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
        
        match lex.peek_k()? {
          (WK::Semicolon, _) => { lex.bump()?; ctn.push(ex_id); },
          
          (WK::BraceR, _) => { expr = Some(ex_id); }
          
          (_, c) => {
            if cre.get(ex_id).is_like_blok() {
              ctn.push(ex_id);
            } else {
              c.panic_kind(WK::Semicolon)?;
            }
          }
        }
      }

      (cre.extra(&ctn), expr)
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Block{ label, rng, expr }
    };

    Ok(cre.push(this))
  }

}


// Context
pub fn pre_unsafe(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;


  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Unsafe(blok)
  };

  Ok(cre.push(this))
}

pub fn pre_relaxed(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let blok = ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?;


  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Relaxed(blok)
  };

  Ok(cre.push(this))
}


// Variable
pub fn pre_let(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  let item = PattParser::read_patt(ctx!(cre, sin, far, lex, sum, side))?;

  let kind = if lex.peek()?.kind() == WK::Colon {
    lex.bump()?;
    Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?)
  } else {
    None
  };

  let init = if lex.peek()?.kind() == WK::Eq {
    lex.bump()?;
    Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?)
  }
  else {
    None
  };


  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Let{ item, kind, init, ism: start.kind() == WK::Var }
  };

  Ok(cre.push(this))
}
