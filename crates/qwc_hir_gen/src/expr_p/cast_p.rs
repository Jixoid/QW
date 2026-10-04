use crate::{ExprLow, hgen::Ctx, TypeLow};

use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_hir::{self as hir, ExprCategory};


// Cast
pub fn low_cast(ctx: &mut Ctx, it: &ast::Expr, expr: ast::ExprId, kind: ast::TypeId) -> Result<hir::ExprId, Message> {
  let expr_hir_id = ExprLow::low(ctx, expr)?;
  let target_hir_ty = TypeLow::low(ctx, kind)?;
  let expr_hir = ctx.cre.get(expr_hir_id);
  let src_ty = expr_hir.ety;

  // Unpack struct type (whether value or &value)
  let (struct_ty, _src_is_ref) = match ctx.cre.get(src_ty).kind {
    hir::TypeKind::Struct(..) => (src_ty, false),
    hir::TypeKind::Ref(sub, _) => match ctx.cre.get(sub).kind {
      hir::TypeKind::Struct(..) => (sub, true),
      _ => (src_ty, true),
    },
    _ => (src_ty, false),
  };

  // Unpack iface type (whether Iface or &Iface)
  let (iface_ty, target_is_ref) = match ctx.cre.get(target_hir_ty).kind {
    hir::TypeKind::Iface(..) => (Some(target_hir_ty), false),
    hir::TypeKind::Ref(sub, _) => match ctx.cre.get(sub).kind {
      hir::TypeKind::Iface(..) => (Some(sub), true),
      _ => (None, true),
    },
    _ => (None, false),
  };

  if let (true, Some(iface_id)) = (matches!(ctx.cre.get(struct_ty).kind, hir::TypeKind::Struct(..)), iface_ty) {
    let search = ctx.type_impls.get(&struct_ty).map(|tyfuns| tyfuns.traits.get(&iface_id)).flatten();

    if let None = search {
      // Check if struct implements iface in O(1) via cache_impl!
      let struct_name = ctx.type_name(struct_ty);
      let iface_name = ctx.type_name(iface_id);
      return Err(Message::error(
        IFACE_NOT_IMPLEMENTED_FOR_TYPE.args(&[&iface_name, &struct_name]),
        Label::new(it.pos, NOT_IMPLEMENTED_FOR_X.args(&[&iface_name, &struct_name])),
      ));
    }
    
    let ret_ty = if target_is_ref {
      target_hir_ty
    } else {
      ctx.tin.ty_ref(ctx.cre, iface_id, false)
    };

    let this = hir::Expr {
      kind: hir::ExprKind::Cast {
        expr: expr_hir_id,
        kind: ret_ty,
      },
      category: ExprCategory::RValue,
      ety: ret_ty,
    };

    return Ok(ctx.cre.push(this));
  }

  // Not a struct-to-iface cast and not same type:
  if src_ty == target_hir_ty {
    return Ok(expr_hir_id);
  }

  let src_name = ctx.type_name(src_ty);
  let target_name = ctx.type_name(target_hir_ty);
  Err(Message::error(
    CANNOT_CAST_X_TO_Y.args(&[&src_name, &target_name]),
    Label::new(it.pos, CANNOT_CAST.args(&[&src_name, &target_name])),
  ))
}
