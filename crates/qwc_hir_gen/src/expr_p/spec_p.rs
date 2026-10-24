/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::Itertools;
use qwc_ast as ast;
use qwc_hir::{self as hir, PushOkApi};
use qwc_diagnostic::{Label, Message, Status::{self, Success}, Summary, msg::*, ret_err};

use crate::{Ctx, ExprLow, FunCtx, Specialization};


pub fn low_specialize(ctx: &mut Ctx, fctx: &FunCtx, sum: &mut Summary, it: &ast::Expr, callee: ast::ExprId, args: ast::AnyRng) -> Status<hir::ExprId> {
  let callee = ExprLow::low(ctx, fctx, sum, callee)?;
  
  enum KindTarget {
    Expr(hir::ExprId),
    Type(hir::TypeId),
  }

  let (generic, kind_target) = match ctx.cre.get(callee).kind {
    hir::ExprKind::GenericRaw { generic, kind } => (generic, KindTarget::Expr(kind)),
    hir::ExprKind::TypeOf { kind } => {
      let hir::TypeKind::GenericRaw { generic, kind } = ctx.get(kind).kind else { panic!("{:#?}", ctx.get(kind).kind) };
      (generic, KindTarget::Type(kind))
    }
    _ => panic!(),
  };

  let hir::ItemKind::GenericNS { args: generic_trait, .. } = ctx.get(generic).kind else { panic!() };


  let expected = ctx.extra_get(generic_trait).collect_vec();
  let received = ctx.src.extra_any_get(args).collect_vec();

  if expected.len() != received.len() {
    ret_err!(sum, Message::error(FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED.args(&[
      &expected.len().to_string(),
      &received.len().to_string(),
    ]), Label::new_pos(it.pos)))
  }

  let spec = match kind_target {
    KindTarget::Expr(expr) => Specialization::spec_expr(ctx, sum, fctx, &expected, &received, generic, expr)?,

    KindTarget::Type(kind) => {
      let spec = Specialization::spec_type(ctx, sum, &expected, &received, generic, kind)?;
    
      hir::Expr {
        kind: hir::ExprKind::TypeOf { kind: spec },
        category: hir::ExprCategory::RValue,
        pos: it.pos,
        ety: ctx.tin.ty_meta(ctx.cre, spec),
      }.push(ctx.cre)
    }
  };

  Success(spec)
}
