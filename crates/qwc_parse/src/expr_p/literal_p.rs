/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Expr, ExprId, ExprKind, IdentSave, PushOkApi};
use qwc_diagnostic::Status::{self, Success};
use qwc_lexer::WK;

use crate::{ExprParser, ctx, expr_p::postfix_p, meta_p::WordCheck, parse::Ctx};


// Nick
pub fn pre_nick(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;
  let name = start.ident(sum, sin, far)?;

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Nick(name)
  }.push(cre);

  if lex.peek(sum)?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start.into(), this) } else { Success(this) }
}

pub fn pre_nick_no_field_init(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;
  let name = start.ident(sum, sin, far)?;

  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Nick(name)
  }.push_ok(cre)
}


// Self
pub fn pre_self_big(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::SelfB()
  }.push(cre);

  if lex.peek(sum)?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start.into(), this) } else { Success(this) }
}

pub fn pre_self_small(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::SelfS()
  }.push_ok(cre)
}


// Const
pub fn pre_bool(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Bool(start.into(), start.kind() == WK::True)
  }.push_ok(cre)
}

pub fn pre_number(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Number(start.into())
  }.push_ok(cre)
}

pub fn pre_number_point(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Float(start.into()),
  }.push_ok(cre)
}

pub fn pre_string(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;
  let sid = start.string_sid(sum, sin, far)?;

  let kind = if lex.str(start).as_bytes()[0] == b'\'' {
    ExprKind::Char(start.into(), sid)
  } else {
    ExprKind::String(start.into(), sid)
  };


  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind,
  }.push_ok(cre)
}


// Primary
pub fn pre_tuple(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;

  // Unit
  if lex.peek(sum)?.kind() == WK::ParenR {
    lex.bump(sum)?;

    // Post
    return Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Unit,
    }.push_ok(cre)
  }
  
  let expr = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
      
  match lex.get_k(sum)? {
    (WK::ParenR, _) => Success(expr), // (X)

    (WK::Comma, _) => { // (X, ...)
      let mut vals = vec![ expr ];
      
      loop {
        if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break }
        
        vals.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);
        
        match lex.get_k(sum)? {
          (WK::ParenR, _) => break,
          (WK::Comma, _) => continue,
          
          (_, c) => c.panic_kind2(sum, WK::Comma, WK::ParenR)?
        }
      }
      
      let rng = cre.extra(&vals);


      // Post
      Expr{
        pos: lex.pos_extend(start),
        kind: ExprKind::Tuple(rng)
      }.push_ok(cre)
    }

    (_, c) => c.panic_kind2(sum, WK::ParenR, WK::Comma)?
  }
}

pub fn pre_propagate(ctx: &mut Ctx) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let start = lex.get(sum)?;
  
  let (rng, prpg) = {
    let mut vals = vec![];
    let mut prpg = None;
    
    loop {
      if lex.peek(sum)?.kind() == WK::BracketR { lex.bump(sum)?; break }
      
      vals.push(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);
      
      match lex.get_k(sum)? {
        (WK::BracketR, _) => break,
        (WK::Comma, _) => continue,

        (WK::Semicolon, _) => {
          prpg = Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?);
          lex.get(sum)?.expect_kind(sum, WK::BracketR)?;
          break
        }
        
        (_, c) => c.panic_kind2(sum, WK::Comma, WK::BracketR)?
      }
    }
    
    (cre.extra(&vals), prpg)
  };


  // Post
  Expr{
    pos: lex.pos_extend(start),
    kind: match prpg {
      None => ExprKind::Array(rng),
      Some(v) => ExprKind::Propagate(rng, v)
    }
  }.push_ok(cre)
}
