use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{Ctx, ExprLow, FunBuilder, RawTerminator, ExprEmit};



pub fn low_return(ctx: &mut Ctx, bbld: &mut FunBuilder, val: Option<hir::ExprId>) -> Result<(), Message> {
  let val = val.map(|id| ExprLow::low(ctx, bbld, id).transpose().unwrap()).transpose()?;

  RawTerminator::Return(val).terminate(bbld);
  
  let dead_bb = bbld.create_block();
  bbld.switch_to(dead_bb);
  
  Ok(())
}

pub fn low_break(ctx: &mut Ctx, bbld: &mut FunBuilder, val: Option<hir::ExprId>) -> Result<(), Message> {
  let frame = bbld.peek_loop().cloned().expect("break used outside of loop");

  if let Some(val_id) = val {
    if let Some(slot) = frame.result_slot {
      if let Some(v) = ExprLow::low(ctx, bbld, val_id)? {
        mir::Expr::Store {
          target: slot.into(),
          kind: frame.result_ty,
          value: v,
        }.emit(bbld);
      }
    }
  }

  RawTerminator::Jump(frame.exit_bb).terminate(bbld);
  
  let dead_bb = bbld.create_block();
  bbld.switch_to(dead_bb);

  Ok(())
}

pub fn low_continue(_ctx: &mut Ctx, bbld: &mut FunBuilder) -> Result<(), Message> {
  let frame = bbld.peek_loop().cloned().expect("continue used outside of loop");

  RawTerminator::Jump(frame.continue_bb).terminate(bbld);
  
  let dead_bb = bbld.create_block();
  bbld.switch_to(dead_bb);

  Ok(())
}
