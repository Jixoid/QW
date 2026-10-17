/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir};
use qwc_ast as ast;

use crate::{Ctx, TypeLow};


pub fn low_option(ctx: &mut Ctx, sum: &mut Summary, kind: ast::TypeId) -> Status<hir::TypeId> {
  let hir_kind = TypeLow::low(ctx, sum, kind)?;

  // Check
  let lay = ctx.get(hir_kind).layout;
  let pos = ctx.src.get(kind).pos;

  if lay.is_dynamic() {
    ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["option"]), Label::new_pos(pos)))
  }

  
  // Post
  Status::Success(ctx.tin.ty_option(ctx.cre, ctx.ideps, hir_kind))
}
