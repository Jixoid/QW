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


// Condition
pub fn low_if(ctx: &mut Ctx, it: &ast::Expr, cond: ast::ExprId, then: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let cond_ast = ctx.src.get(cond);
  let cond = ExprLow::low(ctx, cond)?;
  let cond_ty = (ctx.cre.get(cond) as &hir::Expr).ety;

  if cond_ty != ctx.tin.ty_bool() {
    let found_ty = ctx.type_name(cond_ty);
    return Err(Message::error(
      EXPECTED_BUT_FOUND.args(&["bool", &found_ty]),
      Label::new(cond_ast.pos, EXPECTED_X.args(&["bool"])),
    ));
  }


  let then = ExprLow::low(ctx, then)?;

  let elsb = elsb.map(|id| ExprLow::low(ctx, id)).transpose()?;

  let ety = helper::find_duo_res_ty(ctx, it.pos, then, elsb)?;


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::If { cond, then, elsb },
    category: ExprCategory::RValue,
    ety,
  };

  Ok(ctx.cre.push(this))
}
