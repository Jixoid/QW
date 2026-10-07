/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use inkwell::values::FunctionValue;
use qwc_mir::{Block, BlokId, BlokRng, Inst, Terminator, Type, TypeKind};
use rustc_hash::FxHashMap;

use crate::{
	FnCtx, InstLow, ValueLow,
	cgen::{CtxI, CtxM},
};


pub struct BlokLow;

impl BlokLow {

	pub fn low_fn<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, fv: FunctionValue<'ctx>, ty: &Type, entry: BlokId, blocks: BlokRng) {
		let is_ret_unit = match ty.kind {
			TypeKind::Fun { ret, .. } => {
				let ret_ty: &Type = ictx.cre.get(ret);
				matches!(ret_ty.kind, TypeKind::Unit)
			}
			_ => panic!("Expected function type for function symbol"),
		};

		let mut fctx = FnCtx::new(fv.get_params());

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
