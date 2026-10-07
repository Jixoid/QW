/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, IdentSave};
use qwc_diagnostic::Message;
use qwc_lexer::WK;

use crate::{ExprParser, ctx, expr_p::postfix_p, meta_p::WordCheck, parse::Ctx};


// Nick
pub fn pre_nick(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;
  let name = start.ident(sin, far)?;

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Nick(name)
  };
  let this = cre.push(this);

  if lex.peek()?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start.into(), this) } else { Ok(this) }
}


// Self
pub fn pre_self_big(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::SelfB()
  };

  Ok(cre.push(this))
}

pub fn pre_self_small(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::SelfS()
  };

  Ok(cre.push(this))
}


// Const
pub fn pre_bool(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Bool(start.into(), start.kind() == WK::True)
  };

  Ok(cre.push(this))
}

pub fn pre_number(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Number(start.into())
  };

  Ok(cre.push(this))
}

pub fn pre_string(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  // Post
  let this = Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::String(start.into())
  };

  Ok(cre.push(this))
}


// Primary
pub fn pre_tuple(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;

  // Unit
  if lex.peek()?.kind() == WK::ParenR {
    lex.bump()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Unit,
    };

    return Ok(cre.push(this))
  }
  
  let expr = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
      
  match lex.get_k()? {
    (WK::ParenR, _) => Ok(expr), // (X)

    (WK::Comma, _) => { // (X, ...)
      let mut vals = vec![ expr ];
      
      loop {
        if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break }
        
        vals.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);
        
        match lex.get_k()? {
          (WK::ParenR, _) => break,
          (WK::Comma, _) => continue,
          
          (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?
        }
      }
      
      let rng = cre.extra(&vals);


      // Post
      let this = Expr{
        pos: lex.pos_extend(start),
        kind: ExprKind::Tuple(rng)
      };
      
      Ok(cre.push(this))
    }

    (_, c) => c.panic_kind2(WK::ParenR, WK::Comma)?
  }
}

pub fn pre_propagate(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get()?;
  
  let (rng, prpg) = {
    let mut vals = vec![];
    let mut prpg = None;
    
    loop {
      if lex.peek()?.kind() == WK::BracketR { lex.bump()?; break }
      
      vals.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);
      
      match lex.get_k()? {
        (WK::BracketR, _) => break,
        (WK::Comma, _) => continue,

        (WK::Semicolon, _) => {
          prpg = Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);
          lex.get()?.expect_kind(WK::BracketR)?;
          break
        }
        
        (_, c) => c.panic_kind2(WK::Comma, WK::BracketR)?
      }
    }
    
    (cre.extra(&vals), prpg)
  };


  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: match prpg {
      None => ExprKind::Array(rng),
      Some(v) => ExprKind::Propagate(rng, v)
    }
  };
  
  Ok(cre.push(this))
}
