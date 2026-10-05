use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, ExprCategory};
use qwc_ast as ast;

use crate::{ExprLow, hgen::Ctx, TypeLow};


// Block
pub fn low_block(ctx: &mut Ctx, rng: ast::ExprRng, expr: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  ctx.loc.push_scope();

  let res = (|| {
    let mut stmt = vec![];

    for id in ctx.src.extra_get(rng) {
      let id = ExprLow::low(ctx, id)?;
      stmt.push(id);
    }

    let stmt_rng = ctx.cre.extra(&stmt);
    let expr = expr.map(|id| ExprLow::low(ctx, id)).transpose()?;
    Ok((stmt_rng, expr))
  })();

  ctx.loc.pop_scope();
  
  let (stmt, expr) = res?;

  let ety = expr.map(|id| ctx.cre.get(id).ety).unwrap_or_else(|| ctx.tin.ty_unit() );


  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Block { stmt, expr },
    category: ExprCategory::RValue,
    ety,
  };

  Ok(ctx.cre.push(this))
}


// Variable
pub fn low_let(ctx: &mut Ctx, item: ast::PattId, kind: Option<ast::TypeId>, init: Option<ast::ExprId>, ism: bool, pos: Span) -> Result<hir::ExprId, Message> {
  let (name, span) = match *ctx.src.get(item) {
    ast::Patt::One(ident) => (ident.sid(), ident),

    _ => todo!("patterns in let bindings not implemented yet"),
  };

  let init = init.ok_or_else(|| Message::error(VARIABLE_REQUIRES_INITIALIZER.args(&[ctx.sin.str(name)]), Label::new_pos(pos)))?;
  let init_hir = ExprLow::low(ctx, init)?;
  let init_ty = ctx.cre.get(init_hir).ety;

  let (kind, pos) = if let Some(kind) = kind {
    let init_pos = ctx.src.get(init).pos;
    let kind_pos = ctx.src.get(kind).pos;

    let kind_ty = TypeLow::low(ctx, kind)?;

    if init_ty != kind_ty {
      return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(kind_ty), &ctx.type_name(init_ty)]),
        Label::new(kind_pos, X_DEFINED_HERE.args(&[&ctx.type_name(kind_ty)])))
          .add(Label::new(init_pos, CONFLICTING_DEFINITION))
      );
    }

    (kind_ty, kind_pos)
  } else {
    let init_pos = ctx.src.get(init).pos;

    (init_ty, init_pos)
  };

  
  // Local Push
  let local_id = ctx.loc.insert(name, kind, ism, span);

  // is DST
  if ctx.cre.get(kind).layout.is_dynamic() {
    return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["stack"]), Label::new_pos(pos)))
  }
  
  if ctx.cre.get(kind).layout.is_meta() {
    return Err(Message::error(META_TYPES_CANNOT_EXIST_IN_X.args(&["stack"]), Label::new_pos(pos)))
  }


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Let { local: local_id, init: init_hir},
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_unit(),
  };

  Ok(ctx.cre.push(this))
}
