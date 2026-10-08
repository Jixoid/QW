/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{ExprLow, hgen::Ctx, TypeLow};

use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};


// Cast
pub fn low_cast(ctx: &mut Ctx, it: &ast::Expr, expr: ast::ExprId, target: ast::TypeId) -> Result<hir::ExprId, Message> {
  let expr = ExprLow::low(ctx, expr)?;
  let target = TypeLow::low(ctx, target)?;
  
  let expr_ty = ctx.cre.get(expr).ety;


  // Choose
  let ret: Option<(fn(_, _, _, _, _) -> _, hir::TypeId)> =
  match ctx.get_type(target).kind {
    hir::TypeKind::Ref(target, _) => {

      // Choose
      match ctx.get_type(target).kind {
        hir::TypeKind::Iface(..) => Some((cast_to_iface_ref, target)),
        
        _ => None,
      }
    }
    hir::TypeKind::Trait(..) => Some((cast_to_trait, target)),
    
    _ => None
  };

  match ret {
    Some((fun, target_unwrap)) => fun(ctx, it, expr, target, target_unwrap),
    
    None => Err(Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(expr_ty), &ctx.type_name(target)]),
      Label::new(it.pos, CANNOT_CAST),
    ))
  }
}



fn cast_to_iface_ref(ctx: &mut Ctx, it: &ast::Expr, expr: hir::ExprId, target: hir::TypeId, target_unwrap: hir::TypeId) -> Result<hir::ExprId, Message> {
  let expr_ty = ctx.cre.get(expr).ety;
  
  // Choose
  let expr_unwrap = match ctx.get_type(expr_ty).kind {
    hir::TypeKind::Ref(expr_ty, _) => {
      
      // Choose
      match ctx.get_type(expr_ty).kind {
        hir::TypeKind::Struct(..) => Some(expr_ty),

        _ => None
      }
    }

    _ => None
  }
  .ok_or_else(||
    Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(expr_ty), &ctx.type_name(target)]),
      Label::new(it.pos, CANNOT_CAST),
    )
  )?;


  // Search
  ctx.type_impls.get(&expr_unwrap).map(|tyfuns| tyfuns.traits.get(&target_unwrap)).flatten()
    .ok_or_else(||
      Message::error(X_NOT_IMPLEMENTED_FOR_TYPE.args(&["iface", &ctx.type_name(expr_ty), &ctx.type_name(target)]),
        Label::new_pos(it.pos),
      )
    )?;

  

  // Post
  hir::Expr {
    kind: hir::ExprKind::CastToIfaceRef {
      ref_of_expr: expr,
      ref_of_type: expr_unwrap,
      target_iface: target_unwrap,
    },
    category: ExprCategory::RValue,
    ety: target,
  }.push_ok(ctx.cre)
}

fn cast_to_trait(ctx: &mut Ctx, it: &ast::Expr, expr: hir::ExprId, target: hir::TypeId, _: hir::TypeId) -> Result<hir::ExprId, Message> {
  let expr_ty = ctx.cre.get(expr).ety;
  
  // Choose
  match ctx.get_type(expr_ty).kind {
    hir::TypeKind::Struct(..) => Some(()),

    _ => None
  }
  .ok_or_else(||
    Message::error(
      CANNOT_CAST_X_TO_Y.args(&[&ctx.type_name(expr_ty), &ctx.type_name(target)]),
      Label::new(it.pos, CANNOT_CAST),
    )
  )?;


  // Search
  ctx.type_impls.get(&expr_ty).map(|tyfuns| tyfuns.traits.get(&target)).flatten()
    .ok_or_else(||
      Message::error(X_NOT_IMPLEMENTED_FOR_TYPE.args(&["trait", &ctx.type_name(expr_ty), &ctx.type_name(target)]),
        Label::new_pos(it.pos),
      )
    )?;


  // Static & Hidden
  let hidden = ctx.tin.ty_trait_from(ctx.cre, expr_ty, target);


  // Post
  hir::Expr {
    kind: hir::ExprKind::CastToTrait {
      expr: expr,
      target_trait: target,
    },
    category: ExprCategory::RValue,
    ety: hidden,
  }.push_ok(ctx.cre)
}
