/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{builder::{FunBuilder, RawTerminator}, Ctx, ExprLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low_fn(ctx: &mut Ctx, id: hir::ExprId, is_ret_unit: bool, param_tys: &[mir::TypeId]) -> Result<(mir::BlokId, mir::BlokRng), Message> {
    let mut fbld = FunBuilder::new();

    for (local_id, &param_ty) in param_tys.iter().enumerate() {
      let slot = fbld.build_alloca(param_ty);
      fbld.local_to_alloca.insert(local_id as u32, slot);
      fbld.emit(mir::Expr::Store {
        target: slot,
        kind: param_ty,
        value: mir::Value::Param(local_id as u32),
      });
    }

    match ctx.src.get(id).kind {
      hir::ExprKind::Block{stmt, expr} => {
        for id in ctx.src.extra_get(stmt) {
          ExprLow::low(ctx, &mut fbld, id)?;
        }

        if let Some(expr) = expr {
          let ret = ExprLow::low(ctx, &mut fbld, expr)?;
          if !fbld.is_current_terminated() {
            RawTerminator::Return(ret).terminate(&mut fbld);
          }
        }
      }

      _ => {
        let ret = ExprLow::low(ctx, &mut fbld, id)?;
        if !fbld.is_current_terminated() {
          RawTerminator::Return(ret).terminate(&mut fbld);
        }
      }
    }

    Ok(fbld.finish(ctx.cre, is_ret_unit))
  }

}
