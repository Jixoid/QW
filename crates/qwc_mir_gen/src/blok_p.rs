/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Status::{self, Success}, Summary};
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{builder::{FunBuilder, RawTerminator}, Ctx, ExprLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low_fn(ctx: &mut Ctx, sum: &mut Summary, id: hir::ExprId, is_ret_unit: bool, param_tys: &[mir::TypeId]) -> Status<(mir::BlokId, mir::BlokRng)> {
    let mut fbld = FunBuilder::new();

    // Args to stack
    for (idx, &param_ty) in param_tys.iter().enumerate() {
      let slot = fbld.build_alloca(param_ty);
      fbld.local_to_alloca.insert(idx as u32, slot);
      fbld.emit(mir::Expr::Store {
        target: slot,
        kind: param_ty,
        value: mir::Value::Param(idx as u32),
      });
    }

    match ctx.src.get(id).kind {
      hir::ExprKind::Block{stmt, expr} => {
        for id in ctx.src.extra_get(stmt) {
          ExprLow::low(ctx, sum, &mut fbld, id)?;
        }

        if let Some(expr) = expr {
          let ret = ExprLow::low(ctx, sum, &mut fbld, expr)?;
          if !fbld.is_current_terminated() {
            RawTerminator::Return(ret).terminate(&mut fbld);
          }
        }
      }

      _ => {
        let ret = ExprLow::low(ctx, sum, &mut fbld, id)?;
        if !fbld.is_current_terminated() {
          RawTerminator::Return(ret).terminate(&mut fbld);
        }
      }
    }

    Success(fbld.finish(ctx.cre, is_ret_unit))
  }

}
