/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Value};
use qwc_string_interner::Sid;

use crate::{FunBuilder, Ctx, SymbLow, builder::{ExprEmit, LoopFrame, RawTerminator}, type_p::TypeLow};

mod integer_p;
mod route_p;
mod cast_p;



pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, bbld: &mut FunBuilder, id: hir::ExprId) -> Result<Option<Value>, Message> {
    let it = ctx.src.get(id);

    use hir::ExprKind::*;

    let it = match it.kind {
      // Const
      Const(val) => Some(Self::low_const(ctx, bbld, val)?),

      // MemRef
      GlobalRef(item) => Some(Self::low_global_ref(ctx, item)?),
      LocalRef(local) => Some(Self::low_local_ref(ctx, bbld, it, local)?),

      // Ref & Deref
      Ref(expr) => Some(Self::low_ref(ctx, bbld, expr)?),
      Deref(expr) => Some(Self::low_deref(ctx, bbld, it, expr)?),

      // Variable
      Let{local, init} => {Self::low_let(ctx, bbld, local, init)?; Some(Value::Const(mir::Const::Unit))}
      
      // Block
      Block{stmt, expr} => Self::low_block(ctx, bbld, stmt, expr)?,

      // Assign
      Assign{lhs, rhs} => {Self::low_assign(ctx, bbld, lhs, rhs)?; None},

      // Loop
      Loop{blok, elsb} => Self::low_loop(ctx, bbld, it, blok, elsb)?,

      // Route
      Return(val) => {route_p::low_return(ctx, bbld, val)?; None},
      Break(val) => {route_p::low_break(ctx, bbld, val)?; None},
      Continue => {route_p::low_continue(ctx, bbld)?; None},

      // Branch
      If{cond, then, elsb} => Self::low_if(ctx, bbld, it, cond, then, elsb)?,

      // Call
      Call{callee, args} => Some(Self::low_call(ctx, bbld, callee, args)?),

      // Field
      Field{target, idx} => Some(Self::low_field(ctx, bbld, it, target, idx)?),
      CombinatedInit{kind, fields} => Some(Self::low_combinated_init(ctx, bbld, kind, fields)?),

      // Integer
      IntArithmetic{op, flg, lhs, rhs} => Some(integer_p::low_int_arithmetic(ctx, bbld, op, flg, lhs, rhs)?),
      AssignIntArithmetic{op, flg, lhs, rhs} => {integer_p::low_int_arithmetic_op(ctx, bbld, op, flg, lhs, rhs)?; None},

      IntCondition{op, lhs, rhs} => Some(integer_p::low_int_condition(ctx, bbld, op, lhs, rhs)?),

      // Logic
      BoolLogic{op, lhs, rhs} => Some(Self::low_bool_logic(ctx, bbld, op, lhs, rhs)?),
      BoolNot(val) => Some(Self::low_bool_not(ctx, bbld, val)?),

      // Cast
      CastToIfaceRef{ref_of_expr, ref_of_type, target_iface} => Some(cast_p::low_cast_to_iface_ref(ctx, bbld, ref_of_expr, ref_of_type, target_iface)?),
      CastToTrait{expr, target_trait} => cast_p::low_cast_to_trait(ctx, bbld, expr, target_trait)?,

      c @_ => todo!("{c:#?}")
    };

    Ok(it)
  }


  // Const
  fn low_const(ctx: &mut Ctx, bbld: &mut FunBuilder, val: hir::Const) -> Result<Value, Message> {
    use hir::Const::*;
    
    let this = match val {
      Unit => mir::Const::Unit.into(),
      Bool(b) => mir::Const::Bool(b).into(),
      Int(i) => mir::Const::Int(i).into(),

      Str(sid) => Self::low_const_str(ctx, bbld, sid)?,
    };

    Ok(this)
  }

  fn low_const_str(ctx: &mut Ctx, bbld: &mut FunBuilder, sid: Sid) -> Result<Value, Message> {
    let fatptr = ctx.tin.ty_fatptrint();
    let ptr = ctx.tin.ty_ptr();
    let aint = ctx.tin.ty_arch_int();

    let slot = bbld.build_alloca(fatptr);

    let f0 = bbld.emit(mir::Expr::Gep { target: slot, kind: fatptr, idx: 0 }).unwrap();
    bbld.emit(mir::Expr::Store { target: f0.into(), kind: ptr, value: mir::Const::Str(sid).into() });
    let f1 = bbld.emit(mir::Expr::Gep { target: slot, kind: fatptr, idx: 1 }).unwrap();
    bbld.emit(mir::Expr::Store { target: f1.into(), kind: aint, value: mir::Const::Int(ctx.sin.str(sid).len() as i32 -1).into() });

    let fat_val = bbld.emit(mir::Expr::Load { target: slot, kind: fatptr }).unwrap();
    
    Ok(fat_val.into())
  }


  // MemRef
  fn low_global_ref(ctx: &mut Ctx, item: hir::ItemId) -> Result<Value, Message> {
    let item = SymbLow::low(ctx, item)?.unwrap();


    // Post
    let this = Value::GlobalRef(item);
    
    Ok(this)
  }

  fn low_local_ref(ctx: &mut Ctx, bbld: &mut FunBuilder, it: &hir::Expr, local: u32) -> Result<Value, Message> {
    let target = *bbld.local_to_alloca.get(&local).expect("local variable alloca not found");
    let ty = TypeLow::low(ctx, it.ety)?;
    
    
    // Post
    let this = mir::Expr::Load {
      target,
      kind: ty,
    };

    Ok(bbld.emit(this).unwrap().into())
  }


  // Ref & Deref
  fn low_ref(ctx: &mut Ctx, bbld: &mut FunBuilder, expr: hir::ExprId) -> Result<Value, Message> {
    let target = Self::low_lval(ctx, bbld, expr)?;

    // Post
    Ok(target)
  }

  fn low_deref(ctx: &mut Ctx, bbld: &mut FunBuilder, it: &hir::Expr, expr: hir::ExprId) -> Result<Value, Message> {
    let ptr_val = ExprLow::low(ctx, bbld, expr)?.unwrap();
    let target_ty = TypeLow::low(ctx, it.ety)?;
    let val = bbld.emit(mir::Expr::Load { target: ptr_val, kind: target_ty }).unwrap();
    
    Ok(val.into())
  }


  // Variable
  fn low_let(ctx: &mut Ctx, bbld: &mut FunBuilder, local: u32, init: hir::ExprId) -> Result<(), Message> {
    let init_hir = ctx.src.get(init);
    let ty = TypeLow::low(ctx, init_hir.ety)?;

    let slot = bbld.build_alloca(ty);
    bbld.local_to_alloca.insert(local, slot);

    if let hir::ExprKind::CombinatedInit { kind, fields } = init_hir.kind {
      let kind = TypeLow::low(ctx, kind)?;
      Self::low_combinated_init_into(ctx, bbld, slot, kind, fields)?;
    } else {
      let val = ExprLow::low(ctx, bbld, init)?.unwrap();
      let this = mir::Expr::Store {
        target: slot,
        kind: ty,
        value: val,
      };
      bbld.emit(this);
    }

    Ok(())
  }


  // Block
  fn low_block(ctx: &mut Ctx, bbld: &mut FunBuilder, stmt: hir::ExprRng, expr: Option<hir::ExprId>) -> Result<Option<Value>, Message> {
    for id in ctx.src.extra_get(stmt) {
      ExprLow::low(ctx, bbld, id)?;
    }

    Ok(Some(expr.map(|id| ExprLow::low(ctx, bbld, id)).transpose()?.flatten().unwrap_or(mir::Const::Unit.into())))
  }


  // Assign
  fn low_assign(ctx: &mut Ctx, bbld: &mut FunBuilder, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<(), Message> {
    let target = Self::low_lval(ctx, bbld, lhs)?;
    let rhs_hir = ctx.src.get(rhs);

    if let hir::ExprKind::CombinatedInit { kind, fields } = rhs_hir.kind {
      let kind = TypeLow::low(ctx, kind)?;
      Self::low_combinated_init_into(ctx, bbld, target, kind, fields)?;
    } else {
      let kind = TypeLow::low(ctx, ctx.src.get(lhs).ety)?;
      let value = ExprLow::low(ctx, bbld, rhs)?.unwrap();

      mir::Expr::Store {
        target, 
        kind, 
        value,
      }.emit(bbld);
    }
    
    Ok(())
  }


  // Loop
  fn low_loop(ctx: &mut Ctx, bbld: &mut FunBuilder, it: &hir::Expr, blok: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<Option<Value>, Message> {
    let loop_ty = TypeLow::low(ctx, it.ety)?;
    
    let is_unit_or_never = matches!(ctx.cre.get(loop_ty).kind, mir::TypeKind::Unit);

    let result_slot = (!is_unit_or_never).then(|| bbld.build_alloca(loop_ty));

    let body_bb = bbld.create_block();
    let exit_bb = bbld.create_block();

    // Fallthrough to body_bb
    bbld.terminate(RawTerminator::Jump(body_bb));

    // Push loop frame
    bbld.push_loop(LoopFrame {
      continue_bb: body_bb,
      exit_bb,
      result_slot: result_slot.map(|v| if let Value::SSA(v) = v {v} else { panic!() }),
      result_ty: loop_ty,
    });

    // Lower body
    bbld.switch_to(body_bb);
    let body_val = ExprLow::low(ctx, bbld, blok)?;
    if let (Some(slot), Some(val)) = (result_slot, body_val) {
      bbld.emit(mir::Expr::Store {
        target: slot,
        kind: loop_ty,
        value: val,
      });
    }

    if !bbld.is_current_terminated() {
      bbld.terminate(RawTerminator::Jump(body_bb));
    }

    bbld.pop_loop();

    // If else block is present
    if let Some(elsb) = elsb {
      let else_bb = bbld.create_block();
      bbld.switch_to(else_bb);

      let else_val = ExprLow::low(ctx, bbld, elsb)?;
      if let (Some(slot), Some(val)) = (result_slot, else_val) {
        bbld.emit(mir::Expr::Store {
          target: slot,
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
        target: slot,
        kind: loop_ty,
      });
      Ok(dest.map(Value::SSA))
    } else {
      Ok(Some(Value::Const(mir::Const::Unit)))
    }
  }

  fn low_lval(ctx: &mut Ctx, bbld: &mut FunBuilder, id: hir::ExprId) -> Result<Value, Message> {
    let it = ctx.src.get(id);

    use hir::ExprKind::*;

    match it.kind {
      LocalRef(local) => {
        let target = *bbld.local_to_alloca.get(&local).expect("local variable alloca not found");
        Ok(target)
      }
      
      GlobalRef(item) => {
        Self::low_global_ref(ctx, item)
      }
      
      Field { target, idx } => {
        let target_ptr = Self::low_lval(ctx, bbld, target)?;
        let struct_ty = TypeLow::low(ctx, ctx.src.get(target).ety)?;
        let field_ptr = bbld.emit(mir::Expr::Gep { target: target_ptr, kind: struct_ty, idx }).unwrap();
        Ok(field_ptr.into())
      }
      
      Deref(inner) => {
        let ptr_val = ExprLow::low(ctx, bbld, inner)?.unwrap();
        Ok(ptr_val)
      }
      
      _ => panic!("unexpected lvalue expression: {:?}", it.kind)
    }
  }


  // Branch
  fn low_if(ctx: &mut Ctx, bbld: &mut FunBuilder, it: &hir::Expr, cond: hir::ExprId, then: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<Option<Value>, Message> {
    let if_ty = TypeLow::low(ctx, it.ety)?;

    let is_unit_or_never = matches!(ctx.cre.get(if_ty).kind, mir::TypeKind::Unit);

    let result_slot = (!is_unit_or_never).then(|| bbld.build_alloca(if_ty));

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
        target: slot,
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
          target: slot,
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
        target: slot,
        kind: if_ty,
      });
      Ok(dest.map(Value::SSA))
    } else {
      Ok(Some(Value::Const(mir::Const::Unit)))
    }
  }


  // Call
  fn low_call(ctx: &mut Ctx, bbld: &mut FunBuilder, callee: hir::ExprId, args: hir::ExprRng) -> Result<Value, Message> {
    let callee = ExprLow::low(ctx, bbld, callee)?.unwrap();

    let args = {
      let mut vec = vec![];

      for id in ctx.src.extra_get(args) {
        let it = ExprLow::low(ctx, bbld, id)?.unwrap();
        let id = ctx.cre.push(it);
        vec.push(id);
      }

      ctx.cre.extra(&vec)
    };


    // Post
    let this = mir::Expr::Call {
      callee,
      args,
    }.emit(bbld).unwrap();

    Ok(this.into())
  }


  // Field Create
  fn low_combinated_init_into(ctx: &mut Ctx, bbld: &mut FunBuilder, target_ptr: Value, kind: mir::TypeId, fields: hir::ExprRng) -> Result<(), Message> {
    for (idx, field_id) in ctx.src.extra_get(fields).enumerate() {
      let field_val = ExprLow::low(ctx, bbld, field_id)?.unwrap();
      let field_hir = ctx.src.get(field_id);
      let field_ty = TypeLow::low(ctx, field_hir.ety)?;

      let field_ptr = bbld.emit(mir::Expr::Gep {
        target: target_ptr,
        kind,
        idx: idx as u32,
      }).unwrap();

      bbld.emit(mir::Expr::Store {
        target: field_ptr.into(),
        kind: field_ty,
        value: field_val,
      });
    }

    Ok(())
  }

  fn low_combinated_init(ctx: &mut Ctx, bbld: &mut FunBuilder, kind: hir::TypeId, fields: hir::ExprRng) -> Result<Value, Message> {
    let kind = TypeLow::low(ctx, kind)?;
    let slot = bbld.build_alloca(kind);
    Self::low_combinated_init_into(ctx, bbld, slot, kind, fields)?;

    let val = bbld.emit(mir::Expr::Load {
      target: slot,
      kind,
    }).unwrap();

    Ok(val.into())
  }


  // Field Access
  fn low_field(ctx: &mut Ctx, bbld: &mut FunBuilder, it: &hir::Expr, target: hir::ExprId, idx: u32) -> Result<Value, Message> {
    let target_expr = ctx.src.get(target);
    let struct_ty = TypeLow::low(ctx, target_expr.ety)?;
    let field_ty = TypeLow::low(ctx, it.ety)?;

    let target_ptr = if target_expr.category.is_lvalue() {
      Self::low_lval(ctx, bbld, target)?
    } else {
      let target_val = ExprLow::low(ctx, bbld, target)?.unwrap();
      let slot = bbld.build_alloca(struct_ty);
      bbld.emit(mir::Expr::Store { target: slot, kind: struct_ty, value: target_val });
      slot
    };

    let field_ptr = bbld.emit(mir::Expr::Gep { target: target_ptr, kind: struct_ty, idx }).unwrap();
    let val = bbld.emit(mir::Expr::Load { target: field_ptr.into(), kind: field_ty }).unwrap();
    Ok(val.into())
  }


  // Logic
  fn low_bool_logic(ctx: &mut Ctx, bbld: &mut FunBuilder, op: hir::BoolLogicOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<Value, Message> {
    let kind = TypeLow::low(ctx, (ctx.src.get(lhs) as &hir::Expr).ety)?;
    let lhs = ExprLow::low(ctx, bbld, lhs)?.unwrap();
    let rhs = ExprLow::low(ctx, bbld, rhs)?.unwrap();

    let op = match op {
      hir::BoolLogicOp::And => mir::IntLogicOp::And,
      hir::BoolLogicOp::Or  => mir::IntLogicOp::Or,
      hir::BoolLogicOp::Xor => mir::IntLogicOp::Xor,
    };


    // Post
    let this = mir::Expr::IntLogic {
      op,
      kind,
      lhs,
      rhs
    }.emit(bbld).unwrap();

    Ok(this.into())
  }

  fn low_bool_not(ctx: &mut Ctx, bbld: &mut FunBuilder, val: hir::ExprId) -> Result<Value, Message> {
    let kind = TypeLow::low(ctx, (ctx.src.get(val) as &hir::Expr).ety)?;
    let val = ExprLow::low(ctx, bbld, val)?.unwrap();

    // Post
    let this = mir::Expr::IntUnary {
      op: qwc_mir::IntUnaryOp::Not,
      kind,
      val,
    }.emit(bbld).unwrap();

    Ok(this.into())
  }

}
