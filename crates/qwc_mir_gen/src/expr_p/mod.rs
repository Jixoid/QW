/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Status::{self, Success}, Summary};
use qwc_hir as hir;
use qwc_mir::{self as mir, PushOkApi, Value};
use qwc_string_interner::Sid;

use crate::{FunBuilder, Ctx, SymbLow, builder::{ExprEmit, LoopFrame, RawTerminator}, type_p::TypeLow};

mod integer_p;
mod route_p;
mod float_p;
mod cast_p;



pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, id: hir::ExprId) -> Status<Option<Value>> {
    let it = ctx.src.get(id);

    use hir::ExprKind::*;

    let it = match it.kind {
      // Const
      Const(val) => Some(Self::low_const(ctx, sum, bbld, val)?),

      // MemRef
      GlobalRef(item) => Some(Self::low_global_ref(ctx, sum, item)?),
      LocalRef(local) => Some(Self::low_local_ref(ctx, sum, bbld, it, local)?),

      // Ref & Deref
      Ref(expr, _) => Some(Self::low_ref(ctx, sum, bbld, expr)?),
      Deref(expr) => Some(Self::low_deref(ctx, sum, bbld, it, expr)?),

      // Variable
      Let{local, init} => {Self::low_let(ctx, sum, bbld, local, init)?; Some(Value::Const(mir::Const::Unit))}
      
      // Block
      Block{stmt, expr} => Self::low_block(ctx, sum, bbld, stmt, expr)?,

      // Assign
      Assign{lhs, rhs} => {Self::low_assign(ctx, sum, bbld, lhs, rhs)?; None},

      // Loop
      Loop{blok, elsb} => Self::low_loop(ctx, sum, bbld, it, blok, elsb)?,

      // Route
      Return(val) => {route_p::low_return(ctx, sum, bbld, val)?; None},
      Break(val) => {route_p::low_break(ctx, sum, bbld, val)?; None},
      Continue => {route_p::low_continue(ctx, sum, bbld)?; None},

      // Branch
      If{cond, then, elsb} => Self::low_if(ctx, sum, bbld, it, cond, then, elsb)?,

      // Call
      Call{callee, args} => Some(Self::low_call(ctx, sum, bbld, callee, args)?),

      // Field
      Field{target, idx} => Some(Self::low_field(ctx, sum, bbld, it, target, idx)?),
      CombinatedInit{kind, fields} => Some(Self::low_combinated_init(ctx, sum, bbld, kind, fields)?),

      // Integer
      IntArithmetic{op, flg, lhs, rhs} => Some(integer_p::low_int_arithmetic(ctx, sum, bbld, op, flg, lhs, rhs)?),
      AssignIntArithmetic{op, flg, lhs, rhs} => {integer_p::low_int_arithmetic_op(ctx, sum, bbld, op, flg, lhs, rhs)?; None},

      IntCondition{op, lhs, rhs} => Some(integer_p::low_int_condition(ctx, sum, bbld, op, lhs, rhs)?),

      // Float
      FloatArithmetic{op, lhs, rhs} => Some(float_p::low_float_arithmetic(ctx, sum, bbld, op, lhs, rhs)?),

      // Logic
      BoolLogic{op, lhs, rhs} => Some(Self::low_bool_logic(ctx, sum, bbld, op, lhs, rhs)?),
      BoolNot(val) => Some(Self::low_bool_not(ctx, sum, bbld, val)?),

      // Cast
      IntCast(expr) => Some(cast_p::low_cast_int(ctx, sum, bbld, it, expr)?),

      CastToIfaceRef{ref_of_expr, ref_of_type, target_iface} => Some(cast_p::low_cast_to_iface_ref(ctx, sum, bbld, ref_of_expr, ref_of_type, target_iface)?),
      CastToTrait{expr, target_trait} => cast_p::low_cast_to_trait(ctx, sum, bbld, expr, target_trait)?,

      c @_ => todo!("{c:#?}")
    };

    Success(it)
  }


  // Const
  fn low_const(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, val: hir::Const) -> Status<Value> {
    use hir::Const::*;
    
    let this = match val {
      Unit => mir::Const::Unit.into(),
      
      Bool(b) => mir::Const::Bool(b).into(),
      Char(c) => mir::Const::I32(c.to_u32()).into(),
      
      U8(i) => mir::Const::I8(i).into(),
      U16(i) => mir::Const::I16(i).into(),
      U32(i) => mir::Const::I32(i).into(),
      U64(i) => mir::Const::I64(i).into(),
      U128(i) => mir::Const::I128(i).into(),
      
      I8(i) => mir::Const::I8(i as u8).into(),
      I16(i) => mir::Const::I16(i as u16).into(),
      I32(i) => mir::Const::I32(i as u32).into(),
      I64(i) => mir::Const::I64(i as u64).into(),
      I128(i) => mir::Const::I128(i as u128).into(),
      
      F16(fp) => mir::Const::F16(fp).into(),
      F32(fp) => mir::Const::F32(fp).into(),
      F64(fp) => mir::Const::F64(fp).into(),
      F128(fp) => mir::Const::F128(fp).into(),

      Array(kind, rng) => Self::low_const_array(ctx, sum, bbld, kind, rng)?,
      
      Str(sid) => Self::low_const_str(ctx, sum, bbld, sid)?,
    };

    Success(this)
  }

  fn low_const_str(ctx: &mut Ctx, _sum: &mut Summary, bbld: &mut FunBuilder, sid: Sid) -> Status<Value> {
    let fatptr = ctx.tin.ty_fatptrint();
    let ptr = ctx.tin.ty_ptr();
    let aint = ctx.tin.ty_arch_int();

    let slot = bbld.build_alloca(fatptr);

    let f0 = bbld.emit(mir::Expr::Gep { target: slot, kind: fatptr, idx: 0 }).unwrap();
    bbld.emit(mir::Expr::Store { target: f0.into(), kind: ptr, value: mir::Const::Str(sid).into() });
    let f1 = bbld.emit(mir::Expr::Gep { target: slot, kind: fatptr, idx: 1 }).unwrap();
    bbld.emit(mir::Expr::Store { target: f1.into(), kind: aint, value: mir::Const::I32(ctx.sin.str(sid).len() as u32 -1).into() });

    let fat_val = bbld.emit(mir::Expr::Load { target: slot, kind: fatptr }).unwrap();
    
    Success(fat_val.into())
  }

  fn low_const_array(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, kind: hir::TypeId, rng: hir::ExprRng) -> Status<Value> {
    let kind = TypeLow::low(ctx, sum, kind)?;

    let rng = {
      let mut vec = vec![];

      for id in ctx.src.extra_get(rng) {
        let it = ExprLow::low(ctx, sum, bbld, id)?.unwrap();
        vec.push(it.push(ctx.cre) as mir::ValueId);
      }

      ctx.cre.extra(&vec)
    };

    let id = mir::Const::Array {
      kind,
      rng
    }.into();

    Success(id)
  }


  // MemRef
  fn low_global_ref(ctx: &mut Ctx, sum: &mut Summary, item: hir::ItemId) -> Status<Value> {
    let item = SymbLow::low(ctx, sum, item)?.unwrap();


    // Post
    let this = Value::GlobalRef(item);
    
    Success(this)
  }

  fn low_local_ref(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, it: &hir::Expr, local: u32) -> Status<Value> {
    let target = *bbld.local_to_alloca.get(&local).expect("local variable alloca not found");
    let ty = TypeLow::low(ctx, sum, it.ety)?;
    
    
    // Post
    let this = mir::Expr::Load {
      target,
      kind: ty,
    };

    Success(bbld.emit(this).unwrap().into())
  }


  // Ref & Deref
  fn low_ref(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, expr: hir::ExprId) -> Status<Value> {
    let target = Self::low_lval(ctx, sum, bbld, expr)?;

    // Post
    Success(target)
  }

  fn low_deref(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, it: &hir::Expr, expr: hir::ExprId) -> Status<Value> {
    let ptr_val = ExprLow::low(ctx, sum, bbld, expr)?.unwrap();
    let target_ty = TypeLow::low(ctx, sum, it.ety)?;
    let val = bbld.emit(mir::Expr::Load { target: ptr_val, kind: target_ty }).unwrap();
    
    Success(val.into())
  }


  // Variable
  fn low_let(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, local: u32, init: hir::ExprId) -> Status<()> {
    let init_hir = ctx.src.get(init);
    let ty = TypeLow::low(ctx, sum, init_hir.ety)?;

    let slot = bbld.build_alloca(ty);
    bbld.local_to_alloca.insert(local, slot);

    if let hir::ExprKind::CombinatedInit { kind, fields } = init_hir.kind {
      let kind = TypeLow::low(ctx, sum, kind)?;
      Self::low_combinated_init_into(ctx, sum, bbld, slot, kind, fields)?;
    } else {
      let val = ExprLow::low(ctx, sum, bbld, init)?.unwrap();
      let this = mir::Expr::Store {
        target: slot,
        kind: ty,
        value: val,
      };
      bbld.emit(this);
    }

    Success(())
  }


  // Block
  fn low_block(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, stmt: hir::ExprRng, expr: Option<hir::ExprId>) -> Status<Option<Value>> {
    for id in ctx.src.extra_get(stmt) {
      ExprLow::low(ctx, sum, bbld, id)?;
    }

    let ret = match expr {
      Some(id) => ExprLow::low(ctx, sum, bbld, id)?,
      None => None,
    };

    Success(Some(ret.unwrap_or_else(|| mir::Const::Unit.into())))
  }


  // Assign
  fn low_assign(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<()> {
    let target = Self::low_lval(ctx, sum, bbld, lhs)?;
    let rhs_hir = ctx.src.get(rhs);

    if let hir::ExprKind::CombinatedInit { kind, fields } = rhs_hir.kind {
      let kind = TypeLow::low(ctx, sum, kind)?;
      Self::low_combinated_init_into(ctx, sum, bbld, target, kind, fields)?;
    } else {
      let kind = TypeLow::low(ctx, sum, ctx.src.get(lhs).ety)?;
      let value = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();

      mir::Expr::Store {
        target, 
        kind, 
        value,
      }.emit(bbld);
    }
    
    Success(())
  }


  // Loop
  fn low_loop(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, it: &hir::Expr, blok: hir::ExprId, elsb: Option<hir::ExprId>) -> Status<Option<Value>> {
    let loop_ty = TypeLow::low(ctx, sum, it.ety)?;
    
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
      result_slot: result_slot.map(|v| if let Value::SSA(v) = v {v} else { unreachable!() }),
      result_ty: loop_ty,
    });

    // Lower body
    bbld.switch_to(body_bb);
    let body_val = ExprLow::low(ctx, sum, bbld, blok)?;
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

      let else_val = ExprLow::low(ctx, sum, bbld, elsb)?;
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
      Success(dest.map(Value::SSA))
    } else {
      Success(Some(Value::Const(mir::Const::Unit)))
    }
  }

  pub fn low_lval(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, id: hir::ExprId) -> Status<Value> {
    let it = ctx.src.get(id);

    use hir::ExprKind::*;

    match it.kind {
      LocalRef(local) => {
        let target = *bbld.local_to_alloca.get(&local).expect("local variable alloca not found");
        Success(target)
      }
      
      GlobalRef(item) => {
        Self::low_global_ref(ctx, sum, item)
      }
      
      Field { target, idx } => {
        let target_ptr = Self::low_lval(ctx, sum, bbld, target)?;
        let struct_ty = TypeLow::low(ctx, sum, ctx.src.get(target).ety)?;
        let field_ptr = bbld.emit(mir::Expr::Gep { target: target_ptr, kind: struct_ty, idx }).unwrap();
        Success(field_ptr.into())
      }
      
      Deref(inner) => {
        let ptr_val = ExprLow::low(ctx, sum, bbld, inner)?.unwrap();
        Success(ptr_val)
      }

      _ => panic!("unexpected lvalue expression: {:?}", it.kind)
    }
  }


  // Branch
  fn low_if(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, it: &hir::Expr, cond: hir::ExprId, then: hir::ExprId, elsb: Option<hir::ExprId>) -> Status<Option<Value>> {
    let if_ty = TypeLow::low(ctx, sum, it.ety)?;

    let is_unit_or_never = matches!(ctx.cre.get(if_ty).kind, mir::TypeKind::Unit);

    let result_slot = (!is_unit_or_never).then(|| bbld.build_alloca(if_ty));

    let cond_val = ExprLow::low(ctx, sum, bbld, cond)?.expect("condition must produce a value");

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
    let then_val = ExprLow::low(ctx, sum, bbld, then)?;
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
      let else_val = ExprLow::low(ctx, sum, bbld, elsb_id)?;
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
      Success(dest.map(Value::SSA))
    } else {
      Success(Some(Value::Const(mir::Const::Unit)))
    }
  }


  // Call
  fn low_call(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, callee: hir::ExprId, args: hir::ExprRng) -> Status<Value> {
    let callee = ExprLow::low(ctx, sum, bbld, callee)?.unwrap();

    let args = {
      let mut vec = vec![];

      for id in ctx.src.extra_get(args) {
        let it = ExprLow::low(ctx, sum, bbld, id)?.unwrap();
        vec.push(it.push(ctx.cre));
      }

      ctx.cre.extra(&vec)
    };


    // Post
    let this = mir::Expr::Call {
      callee,
      args,
    }.emit(bbld).unwrap();

    Success(this.into())
  }


  // Field Create
  fn low_combinated_init_into(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, target_ptr: Value, kind: mir::TypeId, fields: hir::ExprRng) -> Status<()> {
    for (idx, field_id) in ctx.src.extra_get(fields).enumerate() {
      let field_val = ExprLow::low(ctx, sum, bbld, field_id)?.unwrap();
      let field_hir = ctx.src.get(field_id);
      let field_ty = TypeLow::low(ctx, sum, field_hir.ety)?;

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

    Success(())
  }

  fn low_combinated_init(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, kind: hir::TypeId, fields: hir::ExprRng) -> Status<Value> {
    let kind = TypeLow::low(ctx, sum, kind)?;
    let slot = bbld.build_alloca(kind);
    Self::low_combinated_init_into(ctx, sum, bbld, slot, kind, fields)?;

    let val = bbld.emit(mir::Expr::Load {
      target: slot,
      kind,
    }).unwrap();

    Success(val.into())
  }


  // Field Access
  fn low_field(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, it: &hir::Expr, target: hir::ExprId, idx: u32) -> Status<Value> {
    let target_expr = ctx.src.get(target);
    let struct_ty = TypeLow::low(ctx, sum, target_expr.ety)?;
    let field_ty = TypeLow::low(ctx, sum, it.ety)?;

    let target_ptr = if target_expr.category.is_lvalue() {
      Self::low_lval(ctx, sum, bbld, target)?
    } else {
      let target_val = ExprLow::low(ctx, sum, bbld, target)?.unwrap();
      let slot = bbld.build_alloca(struct_ty);
      bbld.emit(mir::Expr::Store { target: slot, kind: struct_ty, value: target_val });
      slot
    };

    let field_ptr = bbld.emit(mir::Expr::Gep { target: target_ptr, kind: struct_ty, idx }).unwrap();
    let val = bbld.emit(mir::Expr::Load { target: field_ptr.into(), kind: field_ty }).unwrap();
    Success(val.into())
  }


  // Logic
  fn low_bool_logic(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, op: hir::BoolLogicOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<Value> {
    let kind = TypeLow::low(ctx, sum, (ctx.src.get(lhs) as &hir::Expr).ety)?;
    let lhs = ExprLow::low(ctx, sum, bbld, lhs)?.unwrap();
    let rhs = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();

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

    Success(this.into())
  }

  fn low_bool_not(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, val: hir::ExprId) -> Status<Value> {
    let kind = TypeLow::low(ctx, sum, (ctx.src.get(val) as &hir::Expr).ety)?;
    let val = ExprLow::low(ctx, sum, bbld, val)?.unwrap();

    // Post
    let this = mir::Expr::IntUnary {
      op: qwc_mir::IntUnaryOp::Not,
      kind,
      val,
    }.emit(bbld).unwrap();

    Success(this.into())
  }

}
