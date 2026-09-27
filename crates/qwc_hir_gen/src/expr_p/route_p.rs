use qwc_diagnostic::Message;
use qwc_hir::{self as hir, ExprCategory};
use qwc_ast as ast;

use crate::{expr_p::ExprLow, hgen::Ctx};


// Route
pub fn low_return(ctx: &mut Ctx, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let val = val.map(|id| ExprLow::low(ctx, id)).transpose()?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Return(val),
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_never(),
  };

  Ok(ctx.cre.push(item))
}

pub fn low_break(ctx: &mut Ctx, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let val = val.map(|id| ExprLow::low(ctx, id)).transpose()?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Break(val),
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_never(),
  };

  Ok(ctx.cre.push(item))
}

pub fn low_continue(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Continue,
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_never(),
  };

  Ok(ctx.cre.push(item))
}
