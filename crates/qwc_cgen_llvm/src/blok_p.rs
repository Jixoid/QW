use inkwell::values::FunctionValue;
use qwc_mir::{Block, BlokId, Inst, InstId, Rng, Terminator, Type, TypeId, TypeKind};

use crate::{context::CGenCtx, fn_ctx::FnCtx, type_p::{any_type_to_basic, TypeLow}, value_p::ValueLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low<'ctx>(cgen: &mut CGenCtx<'ctx, '_>, fv: FunctionValue<'ctx>, ty: &Type, entry: BlokId, blocks: Rng, stack: Rng) {
    let (ret_ty_id, is_ret_unit) = match ty.kind {
      TypeKind::Fun {ret, ..} => {
        let ret_ty: &Type = cgen.cre.get(ret);
        (ret, matches!(ret_ty.kind, TypeKind::Unit))
      }
      _ => panic!("Expected function type for function symbol"),
    };

    let mut fn_ctx = FnCtx::new(fv, ret_ty_id, is_ret_unit);

    let mut bb_map: rustc_hash::FxHashMap<BlokId, inkwell::basic_block::BasicBlock<'ctx>> = rustc_hash::FxHashMap::default();

    let entry_bb = cgen.ctx.append_basic_block(fv, "entry");
    bb_map.insert(entry, entry_bb);

    for (id, kind) in cgen.cre.extra_get(blocks) {
      if kind == qwc_mir::id::NodeKind::Blok {
        let b_id = BlokId::new_from((id, kind));
        if b_id != entry {
          let llvm_bb = cgen.ctx.append_basic_block(fv, &format!("bb_{}", b_id.idx()));
          bb_map.insert(b_id, llvm_bb);
        }
      }
    }

    // Stack
    cgen.builder.position_at_end(entry_bb);
    for (id, kind) in cgen.cre.extra_get(stack) {
      if kind == qwc_mir::id::NodeKind::Type {
        let it: &Type = cgen.cre.get(TypeId::new_from((id, kind)));
        let ty = any_type_to_basic(TypeLow::low(cgen.ctx, cgen.cre, it));
        let ptr = cgen.builder.build_alloca(ty, "").unwrap();
        fn_ctx.stack_slots.push(ptr);
      }
    }

    // Populate each block
    for (id, kind) in cgen.cre.extra_get(blocks) {
      if kind == qwc_mir::id::NodeKind::Blok {
        let b_id = BlokId::new_from((id, kind));
        let blok: &Block = cgen.cre.get(b_id);
        let llvm_bb = bb_map[&b_id];
        cgen.builder.position_at_end(llvm_bb);

        for (inst_id, kind) in cgen.cre.extra_get(blok.insts) {
          if kind == qwc_mir::id::NodeKind::Inst {
            let it: &Inst = cgen.cre.get(InstId::new_from((inst_id, kind)));
            Self::low_inst(cgen, &mut fn_ctx, it);
          }
        }

        match blok.term {
          Terminator::Jump(target) => {
            let target_bb = bb_map[&target];
            cgen.builder.build_unconditional_branch(target_bb).unwrap();
          }
          Terminator::Branch { cond, then_bb, else_bb } => {
            let cond_val = ValueLow::low(cgen, &fn_ctx, &cond).into_int_value();
            let llvm_then = bb_map[&then_bb];
            let llvm_else = bb_map[&else_bb];
            cgen.builder.build_conditional_branch(cond_val, llvm_then, llvm_else).unwrap();
          }
          Terminator::Return(val) => {
            if fn_ctx.is_ret_unit {
              cgen.builder.build_return(None).unwrap();
            } else {
              let ret_val: Option<inkwell::values::BasicValueEnum<'ctx>> = match val {
                None => None,
                Some(val) => Some(ValueLow::low(cgen, &fn_ctx, &val)),
              };
              let ret_ref = ret_val.as_ref().map(|v| v as &dyn inkwell::values::BasicValue);
              cgen.builder.build_return(ret_ref).unwrap();
            }
          }
          Terminator::Unreachable => {
            cgen.builder.build_unreachable().unwrap();
          }
        }

        if llvm_bb.get_terminator().is_none() {
          if fn_ctx.is_ret_unit {
            cgen.builder.build_return(None).unwrap();
          } else {
            cgen.builder.build_unreachable().unwrap();
          }
        }
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
