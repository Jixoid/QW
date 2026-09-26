use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Value};

use crate::{BlockBuilder, Ctx, SymbLow, type_p::TypeLow, builder::{LoopFrame, RawTerminator}};


pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, bbld: &mut BlockBuilder, id: hir::ExprId) -> Result<Option<Value>, Message> {
    let it: &hir::Expr = ctx.src.get(id);

    use hir::ExprKind::*;

    let it = match it.kind {
      // Const
      Const(val) => Some(Self::low_const(ctx, val)?),

      // MemRef
      GlobalRef(item) => Some(Self::low_global_ref(ctx, item)?),
      LocalRef(local) => Some(Self::low_local_ref(ctx, bbld, it, local)?),

      // Variable
      Let{local, init} => {Self::low_let(ctx, bbld, local, init)?; Some(Value::Const(mir::Const::Unit))}
      
      // Block
      Block{stmt, expr} => Self::low_block(ctx, bbld, stmt, expr)?,

      // Assign
      Assign{lhs, rhs} => {Self::low_assign(ctx, bbld, lhs, rhs)?; None},

      // Loop
      Loop{blok, elsb} => Self::low_loop(ctx, bbld, it, blok, elsb)?,

      // Route
      Return(val) => {Self::low_return(ctx, bbld, val)?; None},
      Break(val) => {Self::low_break(ctx, bbld, val)?; None},
      Continue => {Self::low_continue(ctx, bbld)?; None},

      // Branch
      If{cond, then, elsb} => Self::low_if(ctx, bbld, it, cond, then, elsb)?,

      c @_ => todo!("{c:#?}")
    };

    Ok(it)
  }


  // Const
  fn low_const(_ctx: &mut Ctx, val: hir::Const) -> Result<Value, Message> {
    let cons = match val {
      hir::Const::Unit => mir::Const::Unit,
      hir::Const::Bool(b) => mir::Const::Bool(b),
      hir::Const::Int(i) => mir::Const::Int(i),
    };

    let this = Value::Const(cons);
    Ok(this)
  }


  // MemRef
  fn low_global_ref(ctx: &mut Ctx, item: hir::ItemId) -> Result<Value, Message> {
    let item = SymbLow::low(ctx, item)?.unwrap();


    // Post
    let this = Value::GlobalRef(
      item
    );
    
    Ok(this)
  }

  fn low_local_ref(ctx: &mut Ctx, bbld: &mut BlockBuilder, it: &hir::Expr, local: u32) -> Result<Value, Message> {
    let slot = *bbld.local_to_slot.get(&local).expect("local variable stack slot not found");
    let ty = TypeLow::low(ctx, it.ety)?;
    
    
    // Post
    let this = mir::Expr::Load {
      target: Value::StackRef(slot),
      kind: ty,
    };

    Ok(Value::SSA(bbld.emit(this).unwrap()))
  }


  // Variable
  fn low_let(ctx: &mut Ctx, bbld: &mut BlockBuilder, local: u32, init: hir::ExprId) -> Result<(), Message> {
    let init_hir: &hir::Expr = ctx.src.get(init);
    let ty = TypeLow::low(ctx, init_hir.ety)?;
    let slot = bbld.alloc_stack(ty);
    bbld.local_to_slot.insert(local, slot);
    let val = ExprLow::low(ctx, bbld, init)?.unwrap();


    // Post
    let this = mir::Expr::Store {
      target: Value::StackRef(slot),
      kind: ty,
      value: val,
    };
    
    bbld.emit(this);
    Ok(())
  }


  // Block
  fn low_block(ctx: &mut Ctx, bbld: &mut BlockBuilder, stmt: hir::Rng, expr: Option<hir::ExprId>) -> Result<Option<Value>, Message> {
    for id in ctx.src.extra_get(stmt) {
      let id = hir::ExprId::new_from(id);
      ExprLow::low(ctx, bbld, id)?;
    }

    if let Some(expr) = expr {
      ExprLow::low(ctx, bbld, expr)
    } else {
      Ok(Some(Value::Const(mir::Const::Unit)))
    }
  }


  // Assign
  fn low_assign(ctx: &mut Ctx, bbld: &mut BlockBuilder, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<(), Message> {
    let target = Self::low_lval(ctx, bbld, lhs)?;
    let kind = TypeLow::low(ctx, (ctx.src.get(lhs) as &hir::Expr).ety)?;
    let value = ExprLow::low(ctx, bbld, rhs)?.unwrap();


    // Post
    let this = mir::Expr::Store{
      target,
      kind,
      value,
    };
    
    bbld.emit(this);
    Ok(())
  }


  // Loop
  fn low_loop(ctx: &mut Ctx, bbld: &mut BlockBuilder, it: &hir::Expr, blok: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<Option<Value>, Message> {
    let loop_ty = TypeLow::low(ctx, it.ety)?;
    let is_unit_or_never = {
      let ty: &mir::Type = ctx.cre.get(loop_ty);
      matches!(ty.kind, mir::TypeKind::Unit)
    };

    let result_slot = if !is_unit_or_never {
      Some(bbld.alloc_stack(loop_ty))
    } else {
      None
    };

    let body_bb = bbld.create_block();
    let exit_bb = bbld.create_block();

    // Fallthrough to body_bb
    bbld.terminate(RawTerminator::Jump(body_bb));

    // Push loop frame
    bbld.push_loop(LoopFrame {
      continue_bb: body_bb,
      exit_bb,
      result_slot,
      result_ty: loop_ty,
    });

    // Lower body
    bbld.switch_to(body_bb);
    let body_val = ExprLow::low(ctx, bbld, blok)?;
    if let (Some(slot), Some(val)) = (result_slot, body_val) {
      bbld.emit(mir::Expr::Store {
        target: Value::StackRef(slot),
        kind: loop_ty,
        value: val,
      });
    }

    if !bbld.is_current_terminated() {
      bbld.terminate(RawTerminator::Jump(body_bb));
    }

    bbld.pop_loop();

    // If else block is present
    if let Some(elsb_id) = elsb {
      let else_bb = bbld.create_block();
      bbld.switch_to(else_bb);
      let else_val = ExprLow::low(ctx, bbld, elsb_id)?;
      if let (Some(slot), Some(val)) = (result_slot, else_val) {
        bbld.emit(mir::Expr::Store {
          target: Value::StackRef(slot),
          kind: loop_ty,
          value: val,
        });
      }
      if !bbld.is_current_terminated() {
        bbld.terminate(RawTerminator::Jump(exit_bb));
      }
    }

    // Switch to exit_bb
    bbld.switch_to(exit_bb);

    if let Some(slot) = result_slot {
      let dest = bbld.emit(mir::Expr::Load {
        target: Value::StackRef(slot),
        kind: loop_ty,
      });
      Ok(dest.map(Value::SSA))
    } else {
      Ok(Some(Value::Const(mir::Const::Unit)))
    }
  }

  fn low_lval(ctx: &mut Ctx, bbld: &mut BlockBuilder, id: hir::ExprId) -> Result<Value, Message> {
    let it: &hir::Expr = ctx.src.get(id);
    match it.kind {
      hir::ExprKind::LocalRef(local) => {
        let slot = *bbld.local_to_slot.get(&local).expect("local variable stack slot not found");
        Ok(Value::StackRef(slot))
      }
      hir::ExprKind::GlobalRef(item) => {
        Self::low_global_ref(ctx, item)
      }
      _ => {
        panic!("unexpected lvalue expression: {:?}", it.kind);
      }
    }
  }


  // Route
  fn low_return(ctx: &mut Ctx, bbld: &mut BlockBuilder, val: Option<hir::ExprId>) -> Result<(), Message> {
    let val = match val {
      None => None,
      Some(val) => Some(ExprLow::low(ctx, bbld, val)?.unwrap()),
    };

    bbld.terminate(RawTerminator::Return(val));
    let dead_bb = bbld.create_block();
    bbld.switch_to(dead_bb);
    Ok(())
  }

  fn low_break(ctx: &mut Ctx, bbld: &mut BlockBuilder, val: Option<hir::ExprId>) -> Result<(), Message> {
    let frame = bbld.peek_loop().cloned();
    let frame = match frame {
      Some(f) => f,
      None => panic!("break used outside of loop"),
    };

    if let Some(val_id) = val {
      if let Some(slot) = frame.result_slot {
        if let Some(v) = ExprLow::low(ctx, bbld, val_id)? {
          bbld.emit(mir::Expr::Store {
            target: Value::StackRef(slot),
            kind: frame.result_ty,
            value: v,
          });
        }
      }
    }

    bbld.terminate(RawTerminator::Jump(frame.exit_bb));
    let dead_bb = bbld.create_block();
    bbld.switch_to(dead_bb);
    Ok(())
  }

  fn low_continue(_ctx: &mut Ctx, bbld: &mut BlockBuilder) -> Result<(), Message> {
    let frame = bbld.peek_loop().cloned();
    let frame = match frame {
      Some(f) => f,
      None => panic!("continue used outside of loop"),
    };

    bbld.terminate(RawTerminator::Jump(frame.continue_bb));
    let dead_bb = bbld.create_block();
    bbld.switch_to(dead_bb);
    Ok(())
  }


  // Branch
  fn low_if(ctx: &mut Ctx, bbld: &mut BlockBuilder, it: &hir::Expr, cond: hir::ExprId, then: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<Option<Value>, Message> {
    let if_ty = TypeLow::low(ctx, it.ety)?;
    let is_unit_or_never = {
      let ty: &mir::Type = ctx.cre.get(if_ty);
      matches!(ty.kind, mir::TypeKind::Unit)
    };

    let result_slot = if !is_unit_or_never {
      Some(bbld.alloc_stack(if_ty))
    } else {
      None
    };

    let cond_val = ExprLow::low(ctx, bbld, cond)?.expect("condition must produce a value");

    let then_bb = bbld.create_block();
    let merge_bb = bbld.create_block();
    let else_bb = if elsb.is_some() {
      bbld.create_block()
    } else {
      merge_bb
    };

    bbld.terminate(RawTerminator::Branch {
      cond: cond_val,
      then_bb,
      else_bb,
    });

    // Lower then
    bbld.switch_to(then_bb);
    let then_val = ExprLow::low(ctx, bbld, then)?;
    if let (Some(slot), Some(val)) = (result_slot, then_val) {
      bbld.emit(mir::Expr::Store {
        target: Value::StackRef(slot),
        kind: if_ty,
        value: val,
      });
    }
    if !bbld.is_current_terminated() {
      bbld.terminate(RawTerminator::Jump(merge_bb));
    }

    // Lower else if present
    if let Some(elsb_id) = elsb {
      bbld.switch_to(else_bb);
      let else_val = ExprLow::low(ctx, bbld, elsb_id)?;
      if let (Some(slot), Some(val)) = (result_slot, else_val) {
        bbld.emit(mir::Expr::Store {
          target: Value::StackRef(slot),
          kind: if_ty,
          value: val,
        });
      }
      if !bbld.is_current_terminated() {
        bbld.terminate(RawTerminator::Jump(merge_bb));
      }
    }

    // Switch to merge_bb
    bbld.switch_to(merge_bb);

    if let Some(slot) = result_slot {
      let dest = bbld.emit(mir::Expr::Load {
        target: Value::StackRef(slot),
        kind: if_ty,
      });
      Ok(dest.map(Value::SSA))
    } else {
      Ok(Some(Value::Const(mir::Const::Unit)))
    }
  }

}
