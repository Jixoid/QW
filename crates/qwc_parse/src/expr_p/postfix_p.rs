/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{AnyRng, Expr, ExprId, ExprKind, IdentSave, PushOkApi, Thing, ThingKind};
use qwc_diagnostic::{ret_err, Label, Message, Status::{self, Success}, Span, msg::*};
use qwc_lexer::WK;

use crate::{ctx, ExprParser, expr_p::{literal_p, postfix_p}, WordCheck, Ctx, TypeParser};


// Postfix
pub fn post_scope_spec(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> {
  ctx.lex.get(ctx.sum)?;

  match ctx.lex.peek(ctx.sum)?.kind() {
    WK::Lt => post_specialize(ctx, start, lhs),
    _ => post_scope(ctx, start, lhs),
  }
}


pub fn post_member(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get(sum)?;

  let rng = {
    let sub = literal_p::pre_nick(ctx!(cre, sin, far, lex, sum, side))?;
    
    let mut ctn = vec![lhs, sub];

    loop {
      match lex.peek_k(sum)? {
        (WK::Dot, _) => {
          lex.bump(sum)?;
          ctn.push(literal_p::pre_nick(ctx!(cre, sin, far, lex, sum, side))?);
        }

        (WK::Colon2, c) => ret_err!(sum, Message::error(CANNOT_FIELD_ACCESS_AFTER_MEMBER, Label::new_pos(c))),

        _ => break
      }
    }
    
    cre.extra(&ctn)
  };
    

  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::Member(rng)
  }.push_ok(cre)
}

pub fn post_scope(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.peek(sum)?;

  let rng = {
    let sub = literal_p::pre_nick(ctx!(cre, sin, far, lex, sum, side))?;
    
    let mut ctn = vec![lhs, sub];

    loop {
      if lex.peek(sum)?.kind() == WK::Colon2 {
        lex.bump(sum)?;

        if lex.peek(sum)?.kind() == WK::Lt {
          let rng = cre.extra(&ctn);

          let this = Expr{
            pos: lex.pos_extend(start),
            kind: ExprKind::Path(rng)
          }.push(ctx.cre);

          return post_specialize(ctx, start, this);
        }

        ctn.push(literal_p::pre_nick(ctx!(cre, sin, far, lex, sum, side))?);
      } else {
        break
      }
    }
    
    cre.extra(&ctn)
  };
    

  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Path(rng)
  }.push(cre);
  
  if lex.peek(sum)?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start, this) } else { Success(this) }
}


pub fn post_specialize(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get(sum)?;

  let args = if lex.peek(sum)?.kind() == WK::Gt {
    lex.bump(sum)?;
    AnyRng::empty()
  } else {
    let mut args = vec![];

    loop {
      let arg = match lex.peek_k(sum)? {
        (WK::String | WK::Number | WK::BraceL, _) => ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?.to_any(),
        
        _ => TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?.to_any(),
      };
      
      args.push(arg);

      match lex.get_k(sum)? {
        (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::Gt { lex.bump(sum)?; break },
        
        (WK::Gt, _) => break,

        (_, c) => c.panic_kind2(sum, WK::Comma, WK::Gt)?,
      }
    }

    cre.extra_any(&args)
  };

  
  // Post
  let this = Expr{
    pos: lex.pos_extend(start),
    kind: ExprKind::Spec{ callee: lhs, args }
  }.push(cre);

  if lex.peek(sum)?.kind() == WK::BraceL { postfix_p::post_field_create(ctx, start, this) } else { Success(this) }
}


// FieldCreate
pub fn post_field_create(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> { ctx!(ctx => cre, sin, far, lex, sum, side);
  let s = lex.get(sum)?;
  
  let fields = {
    let mut vec = vec![];
    
    loop {
      if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }
      
      let start = lex.peek(sum)?;

      let name = lex.get(sum)?.ident(sum, sin, far)?;
      lex.get(sum)?.expect_kind(sum, WK::Colon)?;
      let expr = ExprParser::read_expr(ctx!(cre,sin,far,lex,sum,side))?;


      vec.push(Thing {
        kind: ThingKind::NamedExpr(name, expr),
        pos: lex.pos_extend(start),
      }.push(cre));

      match lex.get_k(sum)? {
        (WK::Comma, _)  => continue,
        (WK::BraceR, _) => break,

        (_, c) => c.panic_kind2(sum, WK::Comma, WK::BraceR)?,
      }
    }

    cre.extra(&vec)
  };


  // Post
  Expr {
    pos: lex.pos_extend(start),
    kind: ExprKind::FieldCreate { lhs, fields, brace_span: lex.pos_extend(s) }
  }.push_ok(cre)
}


// Cast
pub fn post_cast(ctx: &mut Ctx, start: Span, lhs: ExprId) -> Status<ExprId> {
  ctx.lex.get(ctx.sum)?; // consume 'as'
  let kind = TypeParser::read_type(ctx)?;

  Expr {
    pos: ctx.lex.pos_extend(start),
    kind: ExprKind::Cast { expr: lhs, kind },
  }.push_ok(ctx.cre)
}
