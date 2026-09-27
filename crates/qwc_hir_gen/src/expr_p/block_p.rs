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

  let ety = if let Some(expr) = expr { (ctx.cre.get(expr) as &hir::Expr).ety } else { ctx.tin.ty_unit() };

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
  let patt: &ast::Patt = ctx.src.get(item);
  let (name, span) = match patt {
    ast::Patt::One(ident) => (ident.sid(), (*ident).into()),
    _ => todo!("patterns in let bindings not implemented yet"),
  };

  let init_id = match init {
    Some(id) => id,
    None => {
      let name_str = ctx.sin.str(name);
      return Err(Message::error(VARIABLE_REQUIRES_INITIALIZER
        .args(&[
          name_str
        ]),
        Label::new_pos(pos),
      ));
    }
  };

  let init_hir = ExprLow::low(ctx, init_id)?;
  let init_ty = (ctx.cre.get(init_hir) as &hir::Expr).ety;

  let var_ty = if let Some(kind_id) = kind {
    let declared_ty = TypeLow::low(ctx, kind_id)?;
    if init_ty != declared_ty {
      todo!("{:#?}, {:#?}",
        ctx.cre.get(declared_ty) as &hir::Type,
        ctx.cre.get(init_ty) as &hir::Type,
      );
    }
    declared_ty
  } else {
    init_ty
  };

  let local_id = ctx.loc.insert(name, var_ty, ism, span);

  let this = hir::Expr {
    kind: hir::ExprKind::Let {
      local: local_id,
      init: init_hir,
    },
    category: ExprCategory::lvalue(ism),
    ety: ctx.tin.ty_unit(),
  };

  Ok(ctx.cre.push(this))
}
