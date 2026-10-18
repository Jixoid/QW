/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use qwc_ast as ast;
use qwc_diagnostic::{Status::{self, Success}, Summary};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};

use crate::{Ctx, FunCtx, ExprLow};


// Unary
pub fn low_unary(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, op: ast::UnaryOp, it: &ast::Expr, id: ast::ExprId) -> Status<hir::ExprId> {
  use ast::UnaryOp::*;
  
  match op {
    Deref => low_unary_deref(ctx, fctx, sum, it, id),
    
    Ref{ism} => low_unary_ref(ctx, fctx, sum, it, id, ism),

    Not => low_unary_not(ctx, fctx, sum, it, id),
    
    _ => todo!("{op:#?}")
  }
}

fn low_unary_deref(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, id: ast::ExprId) -> Status<hir::ExprId> {
  let id = ExprLow::low(ctx, fctx, sum, id)?;
  err_bypass!(ctx, it.pos, id);
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
    pos: it.pos,
    ety: ty,
  };

  Success(ctx.cre.push(this))
}

fn low_unary_ref(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, id: ast::ExprId, ism: bool) -> Status<hir::ExprId> {
  let id = ExprLow::low(ctx, fctx, sum, id)?;
  err_bypass!(ctx, ctx.cre.get(id).pos, id);
  
  let ty = ctx.cre.get(id).ety;
  let ty = ctx.tin.ty_ref(ctx.cre, ctx.ideps, ty, ism);

  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Ref(id, ism),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ty,
  };

  Success(ctx.cre.push(this))
}

fn low_unary_not(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, id: ast::ExprId) -> Status<hir::ExprId> {
  let id = ExprLow::low(ctx, fctx, sum, id)?;
  err_bypass!(ctx, it.pos, id);
  let it = ctx.cre.get(id);
  let ty = ctx.get(it.ety);

  use hir::TypeKind::*;

  assert_matches!(ty.kind, Bool{..});


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::BoolNot(id),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: it.ety,
  };

  Success(ctx.cre.push(this))
}
