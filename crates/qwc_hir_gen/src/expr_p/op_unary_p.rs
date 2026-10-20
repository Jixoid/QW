/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, Status::{self, Success}, Summary, msg::*, send_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};

use crate::{Ctx, FunCtx, ExprLow, Infer};


// Unary
pub fn low_unary(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, op: ast::UnaryOp, it: &ast::Expr, id: ast::ExprId) -> Status<hir::ExprId> {
  use ast::UnaryOp::*;
  
  match op {
    Deref => low_unary_deref(ctx, fctx, sum, it, id),
    
    Ref{ism} => low_unary_ref(ctx, fctx, sum, it, id, ism),

    Not => low_unary_not(ctx, fctx, sum, it, id),

    Neg => low_unary_neg(ctx, fctx, sum, it, id),
    Poz => ExprLow::low(ctx, fctx, sum, id),
    
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

fn low_unary_neg(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, id: ast::ExprId) -> Status<hir::ExprId> {
  let val = ExprLow::low(ctx, fctx, sum, id)?;
  err_bypass!(ctx, it.pos, val);

  let val_ety = ctx.cre.get(val).ety;
  let val_ty = ctx.get(val_ety);

  // Primitive integer negation
  if matches!(val_ty.kind, hir::TypeKind::ArchInt(..) | hir::TypeKind::Int(..)) {
    return hir::Expr {
      kind: hir::ExprKind::IntNeg(val),
      category: ExprCategory::RValue,
      pos: it.pos,
      ety: val_ety,
    }.push_ok(ctx.cre);
  }

  // Primitive float negation
  if matches!(val_ty.kind, hir::TypeKind::Float(..)) {
    return hir::Expr {
      kind: hir::ExprKind::FloatNeg(val),
      category: ExprCategory::RValue,
      pos: it.pos,
      ety: val_ety,
    }.push_ok(ctx.cre);
  }

  // Trait lookup: Neg
  let op_trt = ctx.ideps.prim_op().unwrap().op_neg;
  let op_sid = ctx.sin.sid_neg();

  let (op_fun, op_ret) = Infer::lookup_method_with_trait_unary(ctx, sum, op_trt, val, op_sid, it.pos)
    .map_err(||
      send_err!(sum, Message::error(X_NOT_IMPLEMENTED_FOR_TYPE
        .args(&[
          "trait",
          &ctx.type_name(sum, op_trt),
          &ctx.type_name(sum, val_ety),
        ]),
        Label::new_pos(it.pos)
      ))
    )?;

  let op_fun_ty = ctx.cre.get(op_fun).ety;
  let self_kind = if let hir::TypeKind::Fun { self_kind, .. } = ctx.get(op_fun_ty).kind { self_kind } else { None };

  let val_arg = match self_kind {
    Some(self_ty) if matches!(ctx.get(self_ty).kind, hir::TypeKind::Ref(.., true)) => {
      let ty = ctx.tin.ty_ref(ctx.cre, ctx.ideps, val_ety, true);
      hir::Expr {
        kind: hir::ExprKind::Ref(val, true),
        category: ExprCategory::RValue,
        pos: it.pos,
        ety: ty,
      }.push(ctx.cre)
    }
    Some(self_ty) if matches!(ctx.get(self_ty).kind, hir::TypeKind::Ref(.., false)) => {
      let ty = ctx.tin.ty_ref(ctx.cre, ctx.ideps, val_ety, false);
      hir::Expr {
        kind: hir::ExprKind::Ref(val, false),
        category: ExprCategory::RValue,
        pos: it.pos,
        ety: ty,
      }.push(ctx.cre)
    }
    _ => val,
  };

  let args = ctx.cre.extra(&[val_arg]);

  let it = hir::Expr {
    kind: hir::ExprKind::Call { callee: op_fun, args },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: op_ret,
  }.push(ctx.cre);

  Success(it)
}
