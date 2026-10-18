/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_ast as ast;

use crate::{Ctx, ExprLow, TypeLow, TypeMatch, FunCtx};


// Block
pub fn low_block(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, rng: ast::ExprRng, expr: Option<ast::ExprId>) -> Status<hir::ExprId> {
  ctx.loc.push_scope();

  let mut stmt = vec![];
  let mut unreachable = false;

  for id in ctx.src.extra_get(rng) {
    if unreachable {
      let pos = ctx.src.get(id).pos;
      sum.add(Message::warn(UNREACHABLE_STATEMENT.args(&[]), Label::new_pos(pos)));
      break;
    }

    let hir_id = ExprLow::low(ctx, fctx, sum, id);
    if hir_id.is_fail() {
      ctx.loc.pop_scope();
      return Status::Fail;
    }
    let hir_id = hir_id.unwrap();
    stmt.push(hir_id);
    
    if ctx.cre.get(hir_id).ety == ctx.prims.ty_never {
      unreachable = true;
    }
  }

  let stmt_rng = ctx.cre.extra(&stmt);
  
  let (expr_hir, is_unreachable) = if unreachable {
    if let Some(id) = expr {
      let pos = ctx.src.get(id).pos;
      sum.add(Message::warn(UNREACHABLE_STATEMENT.args(&[]), Label::new_pos(pos)));
    }
    (None, true)
  } else {
    let expr_hir = match expr {
      Some(id) => {
        let e = ExprLow::low(ctx, fctx, sum, id);
        if e.is_fail() {
          ctx.loc.pop_scope();
          return Status::Fail;
        }
        Some(e.unwrap())
      }
      None => None,
    };
    (expr_hir, false)
  };

  ctx.loc.pop_scope();

  let ety = if is_unreachable {
    ctx.prims.ty_never
  } else {
    expr_hir.map(|id| ctx.cre.get(id).ety).unwrap_or_else(|| ctx.prims.ty_unit)
  };


  // Post
  hir::Expr{
    kind: hir::ExprKind::Block { stmt: stmt_rng, expr: expr_hir },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety,
  }.push_ok(ctx.cre)
}


// Variable
pub fn low_let(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, item: ast::PattId, kind: Option<ast::TypeId>, init: Option<ast::ExprId>, ism: bool) -> Status<hir::ExprId> {
  let (name, span) = match *ctx.src.get(item) {
    ast::Patt::One(ident) => (ident.sid(), ident),

    _ => todo!("patterns in let bindings not implemented yet"),
  };

  let Some(init) = init else {
    ret_err!(sum, Message::error(VARIABLE_REQUIRES_INITIALIZER.args(&[ctx.sin.str(name)]), Label::new_pos(it.pos)));
  };
  let init_hir = ExprLow::low(ctx, fctx, sum, init)?;
  let init_ty = ctx.cre.get(init_hir).ety;

  let (kind, pos) = if let Some(kind) = kind {
    let kind_pos = ctx.src.get(kind).pos;

    let kind_ty = TypeLow::low(ctx, sum, kind)?;

    if init_ty != kind_ty {
      TypeMatch::matches(ctx, sum, kind_ty, init_ty, ctx.get(init_hir).pos, None)?;
    }

    (kind_ty, kind_pos)
  } else {
    let init_pos = ctx.src.get(init).pos;

    (init_ty, init_pos)
  };

  
  // Local Push
  let local_id = ctx.loc.insert(name, kind, ism, span);

  // is DST
  if ctx.get(kind).layout.is_dynamic() {
    ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["stack"]), Label::new_pos(pos)));
  }
  
  if ctx.get(kind).layout.is_meta() {
    ret_err!(sum, Message::error(META_TYPES_CANNOT_EXIST_IN_X.args(&["stack"]), Label::new_pos(pos)));
  }


  // Post
  hir::Expr {
    kind: hir::ExprKind::Let { local: local_id, init: init_hir},
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ctx.prims.ty_unit,
  }.push_ok(ctx.cre)
}
