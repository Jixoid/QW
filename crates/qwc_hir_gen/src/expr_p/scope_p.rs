/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast as ast;
use qwc_hir as hir;
use qwc_diagnostic::{Status, Summary};

use crate::{Ctx, FunCtx, ExprLow};


pub fn low_unsafe(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, blok: ast::ExprId) -> Status<hir::ExprId> {
  ctx.cmap.unsafe_level += 1;
  let blok = ExprLow::low(ctx, fctx, sum, blok);
  ctx.cmap.unsafe_level -= 1;

  blok
}

pub fn low_relaxed(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, blok: ast::ExprId) -> Status<hir::ExprId> {
  ctx.cmap.relaxed_level += 1;
  let blok = ExprLow::low(ctx, fctx, sum, blok);
  ctx.cmap.relaxed_level -= 1;

  blok
}
