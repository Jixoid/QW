/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Message;
use qwc_ast as ast;
use qwc_hir as hir;

use crate::{Ctx, FunCtx};

macro_rules! err_bypass {
  ($ctx:expr, $($id:expr),+) => {
    $(
      if $ctx.cre.get($id).ety == $ctx.prims.ty_error {
        return Ok(hir::Expr {
          kind: hir::ExprKind::Error,
          category: hir::ExprCategory::RValue,
          ety: $ctx.prims.ty_error,
        }.push($ctx.cre));
      }
    )+
  };
}

mod condition_p;
mod operator_p;
mod resolve_p;
mod route_p;
mod const_p;
mod block_p;
mod loop_p;
mod cast_p;
mod helper;



pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
    if let Some(&id) = ctx.cmap.cache_expr.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);
    
    use ast::ExprKind::*;

    let it = match it.kind {
      // Resolve
      Nick(ident)   => resolve_p::low_nick(ctx, it.pos, ident)?,
      Path(rng) => resolve_p::low_path(ctx, it.pos, rng)?,
      SelfS() => resolve_p::low_self(ctx, it.pos)?,
      SelfB() => resolve_p::low_self_big(ctx, it.pos)?,

      // Const
      Unit => const_p::low_unit(ctx)?,
      Bool(.., v)   => const_p::low_bool(ctx, v)?,
      Number(span)  => const_p::low_number(ctx, span)?,
      String(_, sid) => const_p::low_string(ctx, sid)?,
      
      // Block
      Block{rng, expr, ..} => block_p::low_block(ctx, fctx, rng, expr)?,
      Let{item, kind, init, ism} => block_p::low_let(ctx, fctx, item, kind, init, ism, it.pos)?,
      
      // Condition
      If{cond, then, elsb} => condition_p::low_if(ctx, fctx, it, cond, then, elsb)?,

      // Loop
      Loop{blok, elsb} => loop_p::low_loop(ctx, fctx, it, blok, elsb)?,
      While{cond, blok, elsb} => loop_p::low_while(ctx, fctx, it, cond, blok, elsb)?,

      // Route
      Return{val, ..} => route_p::low_return(ctx, it, fctx, val)?,
      Break{val, ..} => route_p::low_break(ctx, fctx, val)?,
      Continue{..} => route_p::low_continue(ctx)?,

      // Operator
      Unary{op, val} => operator_p::low_unary(ctx, fctx, op, val)?,
      Binary{op, lhs, rhs} => operator_p::low_binary(ctx, fctx, op, lhs, rhs)?,
      
      Assign{lhs, rhs, op_span} => operator_p::low_assign(ctx, fctx, lhs, rhs, op_span)?,
      AssignOp{op, lhs, rhs, op_span} => operator_p::low_assign_op(ctx, fctx, op, lhs, rhs, op_span)?,

      Call{callee, args} => operator_p::low_call(ctx, fctx, it, callee, args)?,

      FieldCreate{lhs, fields, brace_span} => operator_p::low_field_create(ctx, fctx, lhs, fields, brace_span)?,
      Member(rng) => operator_p::low_member(ctx, fctx, rng)?,

      Cast{expr, kind} => cast_p::low_cast(ctx, fctx, it, expr, kind)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_expr.insert(id, it);

    Ok(it)
  }

}
