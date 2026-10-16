/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use qwc_ast as ast;
use qwc_diagnostic::Message;
use qwc_hir::{self as hir, ExprCategory, PushOkApi};

use crate::{Ctx, FunCtx, ExprLow};


// Unary
pub fn low_unary(ctx: &mut Ctx, fctx: &FunCtx, op: ast::UnaryOp, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  use ast::UnaryOp::*;
  
  match op {
    Deref => low_unary_deref(ctx, fctx, id),
    
    Ref{ism} => low_unary_ref(ctx, fctx, id, ism),

    Not => low_unary_not(ctx, fctx, id),
    
    _ => todo!("{op:#?}")
  }
}

fn low_unary_deref(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, fctx, id)?;
  err_bypass!(ctx, id);
  let it = ctx.cre.get(id);
  let ty = ctx.get(it.ety);

  use hir::TypeKind::*;

  let (ty, ism) = match ty.kind {
    Ref(ty, ism) => (ty, ism),

    _ => todo!("{ty:#?}")
  };


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Deref(id),
    category: ExprCategory::lvalue(ism),
    ety: ty,
  };

  Ok(ctx.cre.push(this))
}

fn low_unary_ref(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId, ism: bool) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, fctx, id)?;
  err_bypass!(ctx, id);
  let it = ctx.cre.get(id);

  let ty = ctx.tin.ty_ref(ctx.cre, ctx.ideps, it.ety, ism);

  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Ref(id, ism),
    category: ExprCategory::RValue,
    ety: ty,
  };

  Ok(ctx.cre.push(this))
}

fn low_unary_not(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, fctx, id)?;
  err_bypass!(ctx, id);
  let it = ctx.cre.get(id);
  let ty = ctx.get(it.ety);

  use hir::TypeKind::*;

  assert_matches!(ty.kind, Bool{..});


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::BoolNot(id),
    category: ExprCategory::RValue,
    ety: it.ety,
  };

  Ok(ctx.cre.push(this))
}
