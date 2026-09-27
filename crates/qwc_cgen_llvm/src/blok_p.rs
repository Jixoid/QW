use inkwell::values::FunctionValue;
use qwc_mir::{Block, BlokId, BlokRng, Inst, Terminator, Type, TypeKind, TypeRng};
use rustc_hash::FxHashMap;

use crate::{
	FnCtx, InstLow, TypeLow, ValueLow, any_type_to_basic,
	cgen::{CtxI, CtxM},
};


pub struct BlokLow;

impl BlokLow {

	pub fn low_fn<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, fv: FunctionValue<'ctx>, ty: &Type, entry: BlokId, blocks: BlokRng, stack: TypeRng) {
		let is_ret_unit = match ty.kind {
			TypeKind::Fun { ret, .. } => {
				let ret_ty: &Type = ictx.cre.get(ret);
				matches!(ret_ty.kind, TypeKind::Unit)
			}
			_ => panic!("Expected function type for function symbol"),
		};

		let mut fctx = FnCtx::new();

		let mut bb_map: FxHashMap<BlokId, inkwell::basic_block::BasicBlock<'ctx>> = FxHashMap::default();

		let entry_bb = ictx.ctx.append_basic_block(fv, "entry");
		bb_map.insert(entry, entry_bb);

		for id in ictx.cre.extra_get(blocks) {
			if id != entry {
				let llvm_bb = ictx
					.ctx
					.append_basic_block(fv, &format!("bb_{}", id.idx()));
				bb_map.insert(id, llvm_bb);
			}
		}


		// Stack
		ictx.builder.position_at_end(entry_bb);
		for id in ictx.cre.extra_get(stack) {
			let it: &Type = ictx.cre.get(id);
			let ty = any_type_to_basic(TypeLow::low(ictx, it));
			let ptr = ictx.builder.build_alloca(ty, "").unwrap();
			fctx.stack_slots.push(ptr);
		}

		for (i, param) in fv.get_params().into_iter().enumerate() {
			ictx.builder.build_store(fctx.stack_slots[i], param).unwrap();
		}


		// Populate each block
		for id in ictx.cre.extra_get(blocks) {
			let blok: &Block = ictx.cre.get(id);
			let llvm_bb = bb_map[&id];
			ictx.builder.position_at_end(llvm_bb);

			for id in ictx.cre.extra_get(blok.insts) {
				let it: &Inst = ictx.cre.get(id);

				match it.dest {
					None => InstLow::low_sideff(uctx, ictx, &mut fctx, &it.kind),
					Some(dest) => InstLow::low_result(uctx, ictx, &mut fctx, &it.kind, dest),
				}
			}

			match blok.term {
				Terminator::Jump(target) => {
					let target_bb = bb_map[&target];
					ictx.builder.build_unconditional_branch(target_bb).unwrap();
				}

				Terminator::Branch{cond,then_bb,else_bb} => {
					let cond_val = ValueLow::low(uctx, ictx, &fctx, &cond).into_int_value();
					let llvm_then = bb_map[&then_bb];
					let llvm_else = bb_map[&else_bb];
					ictx.builder
						.build_conditional_branch(cond_val, llvm_then, llvm_else)
						.unwrap();
				}

				Terminator::Return(val) => {
					if is_ret_unit {
						ictx.builder.build_return(None).unwrap();
					} else {
						let ret_val: Option<inkwell::values::BasicValueEnum<'ctx>> = match val {
							None => None,
							Some(val) => Some(ValueLow::low(uctx, ictx, &fctx, &val)),
						};
						let ret_ref = ret_val
							.as_ref()
							.map(|v| v as &dyn inkwell::values::BasicValue);
						ictx.builder.build_return(ret_ref).unwrap();
					}
				}

				Terminator::Unreachable => {
					ictx.builder.build_unreachable().unwrap();
				}
			}

			if llvm_bb.get_terminator().is_none() {
				if is_ret_unit {
					ictx.builder.build_return(None).unwrap();
				} else {
					ictx.builder.build_unreachable().unwrap();
				}
			}
		}
	}

}
