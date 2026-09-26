use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{builder::{FnBuilder, RawTerminator}, Ctx, ExprLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low_fn(ctx: &mut Ctx, id: hir::ExprId, is_ret_unit: bool) -> Result<(mir::BlokId, mir::Rng, mir::Rng), Message> {
    let mut fbld = FnBuilder::new();

    let it: &hir::Expr = ctx.src.get(id);

    match it.kind {
      hir::ExprKind::Block{stmt, expr} => {
        for s_id in ctx.src.extra_get(stmt) {
          let s_id = hir::ExprId::new_from(s_id);
          ExprLow::low(ctx, &mut fbld, s_id)?;
        }

        if let Some(expr) = expr {
          let ret = ExprLow::low(ctx, &mut fbld, expr)?;
          if !fbld.is_current_terminated() {
            fbld.terminate(RawTerminator::Return(ret));
          }
        }
      }
      _ => {
        let ret = ExprLow::low(ctx, &mut fbld, id)?;
        if !fbld.is_current_terminated() {
          fbld.terminate(RawTerminator::Return(ret));
        }
      }
    }

    Ok(fbld.finish(ctx.cre, is_ret_unit))
  }

}
