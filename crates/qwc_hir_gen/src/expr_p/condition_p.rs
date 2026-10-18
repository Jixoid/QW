/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};

use super::helper;
use crate::{ExprLow, Ctx, FunCtx};


// Condition
pub fn low_if(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, cond: ast::ExprId, then: ast::ExprId, elsb: Option<ast::ExprId>) -> Status<hir::ExprId> {
  let cond_ast = ctx.src.get(cond);
  let cond = ExprLow::low(ctx, fctx, sum, cond)?;
  let cond_ty = (ctx.cre.get(cond) as &hir::Expr).ety;

  if cond_ty != ctx.prims.ty_bool {
    let found_ty = ctx.type_name(sum, cond_ty);
    ret_err!(sum, Message::error(
      EXPECTED_BUT_FOUND.args(&["bool", &found_ty]),
      Label::new(cond_ast.pos, EXPECTED_X.args(&["bool"])),
    ));
  }


  let then = ExprLow::low(ctx, fctx, sum, then)?;

  let elsb = match elsb {
    Some(id) => Some(ExprLow::low(ctx, fctx, sum, id)?),
    None => None,
  };

  let ety = helper::find_duo_res_ty(ctx, sum, it.pos, ctx.cre.get(then).ety, elsb.map(|id| ctx.cre.get(id).ety))?;


  // Post
  hir::Expr {
    kind: hir::ExprKind::If { cond, then, elsb },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety,
  }.push_ok(ctx.cre)
}
