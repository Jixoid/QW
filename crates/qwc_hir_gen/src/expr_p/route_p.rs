/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Status::{self, Success}, StatusApi, StatusApi2, Summary, msg::*, ret_err, send_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_ast as ast;

use crate::{ExprLow, FunCtx, expr_p::const_p, hgen::Ctx, TypeMatch};


// Route
pub fn low_return(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, val: Option<ast::ExprId>) -> Status<hir::ExprId> {
  let ret_ty = fctx.ret_ty.status().map_err(||
    send_err!(sum, Message::error(RETURN_OUTSIDE_OF_FUNCTION, Label::new_pos(it.pos)))
  )?;

  let val_hir = val.map(|id| ExprLow::low(ctx, fctx, sum, id)).unwrap_or_else(|| const_p::low_unit(ctx))?;
  let val_ty = ctx.cre.get(val_hir).ety;

  let ret_pos = val.map(|id| ctx.src.get(id).pos).unwrap_or(it.pos);

  TypeMatch::matches_pos(ctx, sum, ret_ty, val_ty, ret_pos)?;
    //.add(Label::new(fctx.sign_pos, DEFINED_HERE))


  // Post
  hir::Expr {
    kind: hir::ExprKind::Return(val_hir),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  }.push_ok(ctx.cre)
}

pub fn low_break(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, val: Option<ast::ExprId>) -> Status<hir::ExprId> {
  let val = val.map(|id| ExprLow::low(ctx, fctx, sum, id)).transpose()?;
  
  let mut lloop = fctx.loops.borrow_mut();
  
  let lloop = lloop.last_mut().status().map_err(||
    send_err!(sum, Message::error(BREAK_OUTSIDE_OF_LOOP, Label::new_pos(it.pos)))
  )?;

  match val {
    None => lloop.opt_ty = true,
    Some(val) => {
      let ety = ctx.cre.get(val).ety;

      if let Some(lty) = lloop.ret_ty {
        ret_err!(sum, Message::error(MISMATCHED_TYPES
          .args(&[
            &ctx.type_name(sum, lty),
            &ctx.type_name(sum, ety),
          ]),
          Label::new_pos(it.pos)
        ))
      } else {
        lloop.ret_ty = Some(ety)
      };
    }
  }


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Break(val),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Success(ctx.cre.push(item))
}

pub fn low_continue(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr) -> Status<hir::ExprId> {
  fctx.loops.borrow().last().status().map_err(||
    send_err!(sum, Message::error(CONTINUE_OUTSIDE_OF_LOOP, Label::new_pos(it.pos)))
  )?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Continue,
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Success(ctx.cre.push(item))
}
