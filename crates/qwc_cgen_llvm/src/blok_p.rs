use inkwell::values::FunctionValue;
use qwc_mir::{Block, Inst, InstId, Type, TypeId, TypeKind};

use crate::{context::CGenCtx, fn_ctx::FnCtx, type_p::{any_type_to_basic, TypeLow}, value_p::ValueLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low<'ctx>(cgen: &mut CGenCtx<'ctx, '_>, fv: FunctionValue<'ctx>, ty: &Type, blok: &Block) {
    let (ret_ty_id, is_ret_unit) = match ty.kind {
      TypeKind::Fun {ret, ..} => {
        let ret_ty: &Type = cgen.cre.get(ret);
        (ret, matches!(ret_ty.kind, TypeKind::Unit))
      }
      _ => panic!("Expected function type for function symbol"),
    };

    let mut fn_ctx = FnCtx::new(fv, ret_ty_id, is_ret_unit);

    let bb = cgen.ctx.append_basic_block(fv, "entry");
    cgen.builder.position_at_end(bb);


    // Stack
    for id in cgen.cre.extra_get(blok.stack) {
      let it: &Type = cgen.cre.get(TypeId::new_from(id));
      let ty = any_type_to_basic(TypeLow::low(cgen.ctx, cgen.cre, it));
      let ptr = cgen.builder.build_alloca(ty, "").unwrap();
      fn_ctx.stack_slots.push(ptr);
    }

    // Insts
    for id in cgen.cre.extra_get(blok.insts) {
      let it: &Inst = cgen.cre.get(InstId::new_from(id));
      Self::low_inst(cgen, &mut fn_ctx, it);
    }

    // Ensure basic block terminator
    if bb.get_terminator().is_none() {
      if fn_ctx.is_ret_unit {
        cgen.builder.build_return(None).unwrap();
      } else {
        cgen.builder.build_unreachable().unwrap();
      }
    }
  }


  fn low_inst<'ctx>(cgen: &mut CGenCtx<'ctx, '_>, fn_ctx: &mut FnCtx<'ctx>, inst: &Inst) {
    match inst.kind {
      qwc_mir::Expr::Store{target, kind: _, value} => {
        let target_val = ValueLow::low(cgen, fn_ctx, &target);
        let ptr = target_val.into_pointer_value();
        let val = ValueLow::low(cgen, fn_ctx, &value);
        cgen.builder.build_store(ptr, val).unwrap();
      }

      qwc_mir::Expr::Load{target, kind} => {
        let target_val = ValueLow::low(cgen, fn_ctx, &target);
        let ptr = target_val.into_pointer_value();
        let elem_ty = TypeLow::low_basic_cached(cgen.ctx, cgen.cre, kind, &mut cgen.type_cache);
        let loaded = cgen.builder.build_load(elem_ty, ptr, "").unwrap();

        if let Some(dest) = inst.dest {
          fn_ctx.ssa_map.insert(dest, loaded);
        }
      }

      qwc_mir::Expr::Return(val) => {
        if fn_ctx.is_ret_unit {
          cgen.builder.build_return(None).unwrap();
        } else {
          let ret_val = ValueLow::low(cgen, fn_ctx, &val);
          cgen.builder.build_return(Some(&ret_val)).unwrap();
        }
      }

      qwc_mir::Expr::Binary(lhs, rhs) => {
        let lhs_val = ValueLow::low(cgen, fn_ctx, &lhs);
        let rhs_val = ValueLow::low(cgen, fn_ctx, &rhs);

        let res = if lhs_val.is_int_value() && rhs_val.is_int_value() {
          cgen.builder.build_int_add(lhs_val.into_int_value(), rhs_val.into_int_value(), "").unwrap().into()
        } else if lhs_val.is_float_value() && rhs_val.is_float_value() {
          cgen.builder.build_float_add(lhs_val.into_float_value(), rhs_val.into_float_value(), "").unwrap().into()
        } else {
          panic!("Unsupported operand types for Binary: {:?}, {:?}", lhs_val, rhs_val);
        };

        if let Some(dest) = inst.dest {
          fn_ctx.ssa_map.insert(dest, res);
        }
      }
    }
  }

}
