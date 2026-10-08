/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, Const, ExprCategory};
use qwc_string_interner::Sid;

use crate::Ctx;


// ZST
pub fn low_unit(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Unit),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_unit,
  };

  Ok(ctx.cre.push(this))
}


// Primitive
pub fn low_bool(ctx: &mut Ctx, val: bool) -> Result<hir::ExprId, Message> {
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Bool(val)),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_bool,
  };

  Ok(ctx.cre.push(this))
}

pub fn low_number(ctx: &mut Ctx, span: Span) -> Result<hir::ExprId, Message> {
  let str = span.str(ctx.far);

  let val = str.parse::<i32>().map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?;


  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Int(val)),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_i32,
  };

  Ok(ctx.cre.push(this))
}

pub fn low_string(ctx: &mut Ctx, sid: Sid) -> Result<hir::ExprId, Message> {
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Str(sid)),
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_ref(ctx.cre, ctx.prims.ty_str, false),
  };

  Ok(ctx.cre.push(this))
}
