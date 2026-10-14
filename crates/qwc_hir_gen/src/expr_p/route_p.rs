/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, msg::*};
use qwc_hir::{self as hir, ExprCategory};
use qwc_ast as ast;

use crate::{ExprLow, FunCtx, expr_p::const_p, hgen::Ctx, TypeMatch};


// Route
pub fn low_return(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let ret_ty = fctx.ret_ty.ok_or_else(|| Message::error(RETURN_OUTSIDE_OF_FUNCTION, Label::new_pos(it.pos)))?;

  let val_hir = val.map(|id| ExprLow::low(ctx, fctx, id)).unwrap_or_else(|| const_p::low_unit(ctx))?;
  let val_ty = ctx.cre.get(val_hir).ety;

  let ret_pos = val.map(|id| ctx.src.get(id).pos).unwrap_or(it.pos);

  TypeMatch::matches_pos(ctx, ret_ty, val_ty, ret_pos).map_err(|err| err
    .add(Label::new(fctx.sign_pos, DEFINED_HERE))
  )?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Return(val_hir),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Ok(ctx.cre.push(item))
}

pub fn low_break(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let val = val.map(|id| ExprLow::low(ctx, fctx, id)).transpose()?;
  
  let mut lloop = fctx.loops.borrow_mut();
  
  let lloop = lloop.last_mut().ok_or_else(||
    Message::error(BREAK_OUTSIDE_OF_LOOP, Label::new_pos(it.pos))
  )?;

  match val {
    None => lloop.opt_ty = true,
    Some(val) => {
      let ety = ctx.cre.get(val).ety;

      if let Some(lty) = lloop.ret_ty {
        return Err(Message::error(MISMATCHED_TYPES
          .args(&[
            &ctx.type_name(lty),
            &ctx.type_name(ety),
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

  Ok(ctx.cre.push(item))
}

pub fn low_continue(ctx: &mut Ctx, it: &ast::Expr, fctx: &FunCtx) -> Result<hir::ExprId, Message> {
  fctx.loops.borrow().last().ok_or_else(||
    Message::error(CONTINUE_OUTSIDE_OF_LOOP, Label::new_pos(it.pos))
  )?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Continue,
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Ok(ctx.cre.push(item))
}
