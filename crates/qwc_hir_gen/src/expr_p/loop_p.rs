/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_hir::{self as hir, ExprCategory};

use super::helper;
use crate::{ExprLow, hgen::Ctx};


// Loop
pub fn low_loop(ctx: &mut Ctx, it: &ast::Expr, blok: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let blok = ExprLow::low(ctx, blok)?;

  let elsb = elsb.map(|id| ExprLow::low(ctx, id)).transpose()?;

  let ety = helper::find_duo_res_ty(ctx, it.pos, blok, elsb)?;


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Loop {blok, elsb},
    category: ExprCategory::RValue,
    ety,
  };
  
  Ok(ctx.cre.push(this))
}

pub fn low_while(ctx: &mut Ctx, it: &ast::Expr, cond: ast::ExprId, blok: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let cond_ast = ctx.src.get(cond);
  let cond = ExprLow::low(ctx, cond)?;
  let cond_ty = ctx.cre.get(cond).ety;

  if cond_ty != ctx.tin.ty_bool() {
    let found_ty = ctx.type_name(cond_ty);
    return Err(Message::error(
      EXPECTED_BUT_FOUND.args(&["bool", &found_ty]),
      Label::new(cond_ast.pos, EXPECTED_X.args(&["bool"])),
    ));
  }

  let blok = ExprLow::low(ctx, blok)?;
  let elsb = elsb.map(|id| ExprLow::low(ctx, id)).transpose()?;
  let ety = helper::find_duo_res_ty(ctx, it.pos, blok, elsb)?;


  // Desugar
  let brk = hir::Expr {
    kind: hir::ExprKind::Break(None),
    category: hir::ExprCategory::RValue,
    ety: ctx.tin.ty_never(),
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