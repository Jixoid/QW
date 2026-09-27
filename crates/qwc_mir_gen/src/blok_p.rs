use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{builder::{FnBuilder, RawTerminator}, Ctx, ExprLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low_fn(ctx: &mut Ctx, id: hir::ExprId, is_ret_unit: bool, param_tys: &[mir::TypeId]) -> Result<(mir::BlokId, mir::BlokRng, mir::TypeRng), Message> {
    let mut fbld = FnBuilder::new();

    for (local_id, &param_ty) in param_tys.iter().enumerate() {
      let slot = fbld.alloc_stack(param_ty);
      fbld.local_to_slot.insert(local_id as u32, slot);
    }

    let it: &hir::Expr = ctx.src.get(id);

    match it.kind {
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
