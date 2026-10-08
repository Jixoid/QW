/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Message;
use qwc_hir::{self as hir, ExprCategory};
use qwc_ast as ast;

use crate::{expr_p::ExprLow, hgen::Ctx};


// Route
pub fn low_return(ctx: &mut Ctx, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let val = val.map(|id| ExprLow::low(ctx, id)).transpose()?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Return(val),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Ok(ctx.cre.push(item))
}

pub fn low_break(ctx: &mut Ctx, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
  let val = val.map(|id| ExprLow::low(ctx, id)).transpose()?;


  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Break(val),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Ok(ctx.cre.push(item))
}

pub fn low_continue(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
  // Post
  let item = hir::Expr {
    kind: hir::ExprKind::Continue,
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_never,
  };

  Ok(ctx.cre.push(item))
}
