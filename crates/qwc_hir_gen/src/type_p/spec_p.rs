/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::Itertools;
use qwc_ast as ast;
use qwc_hir as hir;
use qwc_diagnostic::{Label, Message, msg::*};

use crate::{Ctx, Specialization, TypeLow};


pub fn low_specialize(ctx: &mut Ctx, it: &ast::Type, base: ast::TypeId, args: ast::AnyRng) -> Result<hir::TypeId, Message> {
  let base = TypeLow::low(ctx, base)?;

  let hir::TypeKind::GenericRaw { generic, kind } = ctx.get(base).kind else { panic!() };

  let hir::ItemKind::GenericNS { args: generic_trait, .. } = ctx.get(generic).kind else { panic!() };


  let expected = ctx.extra_get(generic_trait).collect_vec();
  let received = ctx.src.extra_any_get(args).collect_vec();

  if expected.len() != received.len() {
    return Err(Message::error(FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED.args(&[
      &expected.len().to_string(),
      &received.len().to_string(),
    ]), Label::new_pos(it.pos)))
  }

  let spec = Specialization::spec_type(ctx, &expected, &received, generic, kind)?;

  Ok(spec)
}
