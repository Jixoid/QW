/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast as ast;
use qwc_diagnostic::{Applicability, Label, Message, Suggestion, msg::*};
use qwc_hir::{self as hir, ExprCategory};

use super::helper;
use crate::{Ctx, ExprLow, FunCtx, LoopCtx, TypeMatch};


// Loop
pub fn low_loop(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, blok: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let (blok, lctx) = {
    let lctx = LoopCtx{ret_ty: None, opt_ty: false};
    
    fctx.loops.borrow_mut().push(lctx);
    let blok_hir = ExprLow::low(ctx, fctx, blok);
    
    let check = (|| {
      let ast::ExprKind::Block { expr, .. } = ctx.src.get(blok).kind else { unreachable!() };
  
      let Some(expr) = expr else { return Ok(()) };
      let expr_pos = ctx.src.get(expr).pos;
  
      let expr_hir = ExprLow::low(ctx, fctx, expr)?;
      let expr_ty = ctx.cre.get(expr_hir).ety;

      TypeMatch::matches_pos(ctx, ctx.prims.ty_unit, expr_ty, expr_pos).map_err(|msg| msg
        .add(Suggestion::new(YOU_MIGHT_HAVE_MEANT_TO_BREAK_THE_LOOP, Applicability::MaybeIncorrect, format!("{} {}{}", "break", expr_pos.str(ctx.far), ";"), Label::new_pos(expr_pos)))
      )
    })();
    let lctx = fctx.loops.borrow_mut().pop().unwrap();

    check?;
    (blok_hir?, lctx)
  };

  let ety = match (lctx.opt_ty, lctx.ret_ty) {
    (_, None) => ctx.prims.ty_unit,

    (false, Some(ty)) => ty,

    (true, Some(ty)) => ctx.tin.ty_option(ctx.cre, ctx.ideps, ty)
  };

  let elsb = elsb.map(|id| ExprLow::low(ctx, fctx, id)).transpose()?;

  let ety = helper::find_duo_res_ty(ctx, it.pos, ety, elsb.map(|id| ctx.get(id).ety))?;


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Loop {blok, elsb},
    category: ExprCategory::RValue,
    ety,
  };
  
  Ok(ctx.cre.push(this))
}

pub fn low_while(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, cond: ast::ExprId, blok: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let cond_ast = ctx.src.get(cond);
  let cond = ExprLow::low(ctx, fctx, cond)?;
  let cond_ty = ctx.cre.get(cond).ety;

  if cond_ty != ctx.prims.ty_bool {
    let found_ty = ctx.type_name(cond_ty);
    return Err(Message::error(
      EXPECTED_BUT_FOUND.args(&["bool", &found_ty]),
      Label::new(cond_ast.pos, EXPECTED_X.args(&["bool"])),
    ));
  }

  let blok = ExprLow::low(ctx, fctx, blok)?;
  let elsb = elsb.map(|id| ExprLow::low(ctx, fctx, id)).transpose()?;

  let ety = helper::find_duo_res_ty(ctx, it.pos, ctx.cre.get(blok).ety, elsb.map(|id| ctx.cre.get(id).ety))?;


  // Desugar
  let brk = hir::Expr {
    kind: hir::ExprKind::Break(None),
    category: hir::ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };
  let brk = ctx.cre.push(brk);

  let if_expr = hir::Expr {
    kind: hir::ExprKind::If { cond, then: blok, elsb: Some(brk) },
    category: hir::ExprCategory::RValue,
    ety, 
  };
  let if_expr = ctx.cre.push(if_expr);


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Loop { blok: if_expr, elsb },
    category: hir::ExprCategory::RValue,
    ety,
  };
  
  Ok(ctx.cre.push(this))
}
