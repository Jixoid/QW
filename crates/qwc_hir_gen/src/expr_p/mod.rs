use qwc_diagnostic::Message;
use qwc_ast as ast;
use qwc_hir as hir;

use crate::Ctx;

mod condition_p;
mod operator_p;
mod resolve_p;
mod route_p;
mod const_p;
mod block_p;
mod loop_p;
mod helper;



pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
    if let Some(&id) = ctx.cmap.cache_expr.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);
    
    use ast::ExprKind::*;

    let it = match it.kind {
      // Resolve
      Nick(ident)   => resolve_p::low_nick(ctx, it.pos, ident)?,
      Path(rng) => resolve_p::low_path(ctx, it.pos, rng)?,
      SelfS() => resolve_p::low_self(ctx, it.pos)?,

      // Const
      Unit => const_p::low_unit(ctx)?,
      Bool(.., v)  => const_p::low_bool(ctx, v)?,
      Number(span) => const_p::low_number(ctx, span)?,
      
      // Block
      Block{rng, expr, ..} => block_p::low_block(ctx, rng, expr)?,
      Let{item, kind, init, ism} => block_p::low_let(ctx, item, kind, init, ism, it.pos)?,
      
      // Condition
      If{cond, then, elsb} => condition_p::low_if(ctx, it, cond, then, elsb)?,

      // Loop
      Loop{blok, elsb} => loop_p::low_loop(ctx, it, blok, elsb)?,
      While{cond, blok, elsb} => loop_p::low_while(ctx, it, cond, blok, elsb)?,

      // Route
      Return{val, ..} => route_p::low_return(ctx, val)?,
      Break{val, ..} => route_p::low_break(ctx, val)?,
      Continue{..} => route_p::low_continue(ctx)?,

      // Operator
      Unary{op, val} => operator_p::low_unary(ctx, op, val)?,
      Binary{op, lhs, rhs} => operator_p::low_binary(ctx, op, lhs, rhs)?,
      
      Assign{lhs, rhs, op_span} => operator_p::low_assign(ctx, lhs, rhs, op_span)?,
      AssignOp{op, lhs, rhs, op_span} => operator_p::low_assign_op(ctx, op, lhs, rhs, op_span)?,

      Call{callee, args} => operator_p::low_call(ctx, it, callee, args)?,

      FieldCreate{lhs, fields, brace_span} => operator_p::low_field_create(ctx, lhs, fields, brace_span)?,
      Member(rng) => operator_p::low_member(ctx, rng)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_expr.insert(id, it);

    Ok(it)
  }

}
