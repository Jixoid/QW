/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Ctx, ExprLow, FunCtx, TypeLow, TypeMatch};

use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, Status::{self, Success}, StatusApi2, Summary, msg::*, ret_err, send_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};


// Cast
pub fn low_cast(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, expr: ast::ExprId, target: ast::TypeId) -> Status<hir::ExprId> {
  let expr = ExprLow::low(ctx, fctx, sum, expr)?;
  let target = TypeLow::low(ctx, sum, target)?;
  
  let expr_ty = ctx.cre.get(expr).ety;

  // Identity cast
  if expr_ty == target {
    return Success(expr);
  }

  
  // Quick
  let is_int = |ty: hir::TypeId| -> bool { matches!(ctx.get(ty).kind, hir::TypeKind::Int(..) | hir::TypeKind::ArchInt(..)) };
  let is_float = |ty: hir::TypeId| -> bool { matches!(ctx.get(ty).kind, hir::TypeKind::Float(..)) };
  

  // Inter integer cast
  if is_int(expr_ty) && is_int(target) {
    return hir::Expr {
      kind: hir::ExprKind::IntCast(expr),
      category: ExprCategory::RValue,
      pos: it.pos,
      ety: target,
    }.push_ok(ctx.cre);
  }
  
  // Inter float cast
  if is_float(expr_ty) && is_float(target) {
    return hir::Expr {
      kind: hir::ExprKind::FloatCast(expr),
      category: ExprCategory::RValue,
      pos: it.pos,
      ety: target,
    }.push_ok(ctx.cre);  
  }
  
  // float -> int cast
  if is_float(expr_ty) && is_int(target) {
    return hir::Expr {
      kind: hir::ExprKind::FloatToIntCast(expr),
      category: ExprCategory::RValue,
      pos: it.pos,
      ety: target,
    }.push_ok(ctx.cre);  
  }
  
  // int -> float cast
  if is_int(expr_ty) && is_float(target) {
    return hir::Expr {
      kind: hir::ExprKind::IntToFloatCast(expr),
      category: ExprCategory::RValue,
      pos: it.pos,
      ety: target,
    }.push_ok(ctx.cre);  
  }


  // Choose
  let ret: Option<(fn(_, _, _, _, _, _) -> _, hir::TypeId)> =
  match ctx.get(target).kind {
    hir::TypeKind::Ref{kind: target, ..} => {
      match ctx.get(target).kind {
        hir::TypeKind::ItemIface{..} => Some((cast_to_iface_ref, target)),
        
        _ => None,
      }
    }

    hir::TypeKind::Ptr{kind: target, ..} => Some((cast_to_ptr, target)),

    hir::TypeKind::ItemTrait{..} => Some((cast_to_trait, target)),
    
    _ => None
  };

  match ret {
    Some((fun, target_unwrap)) => fun(ctx, sum, it, expr, target, target_unwrap),
    
    None => ret_err!(sum, Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(sum, expr_ty), &ctx.type_name(sum, target)]),
      Label::new(it.pos, CANNOT_CAST),
    ))
  }
}


fn cast_to_iface_ref(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, expr: hir::ExprId, target: hir::TypeId, target_unwrap: hir::TypeId) -> Status<hir::ExprId> {
  let expr_ty = ctx.cre.get(expr).ety;
  
  // Choose
  let expr_unwrap = match ctx.get(expr_ty).kind {
    hir::TypeKind::Ref{kind: expr_ty, ..} => {
      
      // Choose
      match ctx.get(expr_ty).kind {
        hir::TypeKind::Struct(..) | hir::TypeKind::ItemStruct { .. } => Some(expr_ty),

        _ => None
      }
    }

    _ => None
  }
  .status()
  .map_err(||
    send_err!(sum, Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(sum, expr_ty), &ctx.type_name(sum, target)]),
      Label::new(it.pos, CANNOT_CAST),
    ))
  )?;


  // Search
  ctx.type_impls.get(&expr_unwrap).map(|tyfuns| tyfuns.traits.get(&target_unwrap)).flatten().status()
    .map_err(||
      send_err!(sum, Message::error(
        X_NOT_IMPLEMENTED_FOR_TYPE.args(&["iface", &ctx.type_name(sum, target), &ctx.type_name(sum, expr_ty)]),
        Label::new_pos(it.pos),
      ))
    )?;

  

  // Post
  hir::Expr {
    kind: hir::ExprKind::CastToIfaceRef {
      ref_of_expr: expr,
      ref_of_type: expr_unwrap,
      target_iface: target_unwrap,
    },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: target,
  }.push_ok(ctx.cre)
}

fn cast_to_trait(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, expr: hir::ExprId, target: hir::TypeId, _: hir::TypeId) -> Status<hir::ExprId> {
  let expr_ty = ctx.cre.get(expr).ety;
  
  // Choose
  match ctx.get(expr_ty).kind {
    hir::TypeKind::Struct(..) | hir::TypeKind::ItemStruct { .. } => Some(()),

    _ => None
  }
  .status()
  .map_err(||
    send_err!(sum, Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(sum, expr_ty), &ctx.type_name(sum, target)]),
      Label::new(it.pos, CANNOT_CAST),
    ))
  )?;


  // Search
  ctx.type_impls.get(&expr_ty).map(|tyfuns| tyfuns.traits.get(&target)).flatten().status()
    .map_err(||
      send_err!(sum, Message::error(
        X_NOT_IMPLEMENTED_FOR_TYPE.args(&["trait", &ctx.type_name(sum, target), &ctx.type_name(sum, expr_ty)]),
        Label::new_pos(it.pos),
      ))
    )?;


  // Static & Hidden
  let hidden = ctx.tin.ty_trait_from(ctx.cre, expr_ty, target);


  // Post
  hir::Expr {
    kind: hir::ExprKind::CastToTrait {
      expr,
      target_trait: target,
    },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: hidden,
  }.push_ok(ctx.cre)
}


fn cast_to_ptr(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, expr: hir::ExprId, target: hir::TypeId, target_unwrap: hir::TypeId) -> Status<hir::ExprId> {
  let expr_ty = ctx.cre.get(expr).ety;
  
  // Choose
  let expr_unwrap = match ctx.get(expr_ty).kind {
    hir::TypeKind::Ref{kind: expr_ty, ..} => Some(expr_ty),
    _ => None
  }
  .status()
  .map_err(||
    send_err!(sum, Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(sum, expr_ty), &ctx.type_name(sum, target)]),
      Label::new(it.pos, CANNOT_CAST),
    ))
  )?;

  TypeMatch::matches(ctx, sum, target_unwrap, expr_unwrap, it.pos, None)?;


  if ctx.cmap.unsafe_level == 0 {
    ret_err!(sum, Message::error(REQUIRES_UNSAFE_BLOCK, Label::new_pos(it.pos)));
  }


  // Post
  hir::Expr {
    kind: hir::ExprKind::CastToTrait {
      expr,
      target_trait: target,
    },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: target,
  }.push_ok(ctx.cre)
}
