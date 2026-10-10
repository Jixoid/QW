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

use crate::{Ctx, ExprLow, FunCtx, TypeLow, Specialization};


pub fn low_specialize(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, callee: ast::ExprId, args: ast::AnyRng) -> Result<hir::ExprId, Message> {
  let callee = ExprLow::low(ctx, fctx, callee)?;
  
  let hir::ExprKind::GenericRaw { generic, kind } = ctx.cre.get(callee).kind else { panic!() };

  let hir::ItemKind::GenericNS { args: generic_trait, .. } = ctx.cre.get(generic).kind else { panic!() };


  let expected_args = ctx.cre.extra_get(generic_trait).collect_vec();

  let received_args = ctx.src.extra_any_get(args).filter_map(|id| {
    match id.kind() {
      ast::id::NodeKind::Type => {
        let id = ast::TypeId::from_any(id);
        TypeLow::low(ctx, id).map_err(|msg| ctx.sum.add(msg)).ok()
      }

      ast::id::NodeKind::Expr => todo!(),

      _ => panic!(),
    }
  }).collect_vec();


  if expected_args.len() != received_args.len() {
    return Err(Message::error(FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED.args(&[
      &expected_args.len().to_string(),
      &received_args.len().to_string(),
    ]), Label::new_pos(it.pos)))
  }


  // Specialize
  let spec = Specialization{args: &received_args}.spec_expr(ctx, kind);

  Ok(spec)
}
