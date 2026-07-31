use crate::{cgen::ICGen, layout::Target};
use inkwell::context::Context;
use inkwell::types::{AnyTypeEnum, BasicType, StructType};
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;
use inkwell::module::Linkage;
use inkwell::AddressSpace;


pub struct BasicCGen<'a> {
	pub target: &'a Target,
	pub module_type_counter: u32,
}

impl<'a> BasicCGen<'a> {
	pub fn new(target: &'a Target) -> Self {
		Self {
			target,
			module_type_counter: 0,
		}
	}

	fn type_to_llvm<'ctx>(&self, ctx: &'ctx Context, hir_mol: &crate::hir::module::HirModule, ty_id: crate::hir::identy::HirId, structs: &HashMap<crate::hir::identy::HirId, StructType<'ctx>>) -> AnyTypeEnum<'ctx> {
		let ty = hir_mol.get_type(ty_id);
		match &ty.vari {
			crate::hir::types::HirTypeVari::ISize => ctx.i64_type().into(),
			crate::hir::types::HirTypeVari::USize => ctx.i64_type().into(),
			crate::hir::types::HirTypeVari::I8 | crate::hir::types::HirTypeVari::U8 => ctx.i8_type().into(),
			crate::hir::types::HirTypeVari::I16 | crate::hir::types::HirTypeVari::U16 => ctx.i16_type().into(),
			crate::hir::types::HirTypeVari::I32 | crate::hir::types::HirTypeVari::U32 => ctx.i32_type().into(),
			crate::hir::types::HirTypeVari::I64 | crate::hir::types::HirTypeVari::U64 => ctx.i64_type().into(),
			crate::hir::types::HirTypeVari::I128 | crate::hir::types::HirTypeVari::U128 => ctx.i128_type().into(),
			crate::hir::types::HirTypeVari::F16 => ctx.f16_type().into(),
			crate::hir::types::HirTypeVari::F32 => ctx.f32_type().into(),
			crate::hir::types::HirTypeVari::F64 => ctx.f64_type().into(),
			crate::hir::types::HirTypeVari::F128 => ctx.f128_type().into(),
			crate::hir::types::HirTypeVari::Bool => ctx.bool_type().into(),
			crate::hir::types::HirTypeVari::Char => ctx.i32_type().into(),
			crate::hir::types::HirTypeVari::Ptr => ctx.ptr_type(AddressSpace::default()).into(),
			crate::hir::types::HirTypeVari::Void => ctx.void_type().into(),
			crate::hir::types::HirTypeVari::Null => ctx.ptr_type(AddressSpace::default()).into(),
			crate::hir::types::HirTypeVari::PointerOf { .. } => ctx.ptr_type(AddressSpace::default()).into(),
			crate::hir::types::HirTypeVari::ReferenceOf { .. } => ctx.ptr_type(AddressSpace::default()).into(),
			crate::hir::types::HirTypeVari::Function(_) => ctx.ptr_type(AddressSpace::default()).into(),
			crate::hir::types::HirTypeVari::Struct(_) | crate::hir::types::HirTypeVari::Iface(_) => structs.get(&ty_id).unwrap().clone().into(),
			_ => panic!("unimplemented type in cgen: {:?}", ty.vari),
		}
	}
}

impl<'a> ICGen for BasicCGen<'a> {
	fn generate(&mut self, hir_mol: &crate::hir::module::HirModule) -> String {
		let context = Context::create();
		let module = context.create_module(&hir_mol.name);
		let builder = context.create_builder();

		let mut structs: HashMap<crate::hir::identy::HirId, StructType> = HashMap::new();

		// Pass 1: Declare structs
		for (i, t) in hir_mol.list_type.iter().enumerate() {
			let id = crate::hir::identy::HirId::new(crate::hir::identy::HirKind::Type, 0, i as u32);
			if let crate::hir::types::HirTypeVari::Struct(_) = &t.vari {
				let st = context.opaque_struct_type(&format!("type.{}", i));
				structs.insert(id, st);
			} else if let crate::hir::types::HirTypeVari::Iface(_) = &t.vari {
				let st = context.opaque_struct_type(&format!("type.{}", i));
				st.set_body(&[], false);
				structs.insert(id, st);
			}
		}

		// Pass 2: Define struct bodies
		for (i, t) in hir_mol.list_type.iter().enumerate() {
			let id = crate::hir::identy::HirId::new(crate::hir::identy::HirKind::Type, 0, i as u32);
			if let crate::hir::types::HirTypeVari::Struct(s) = &t.vari {
				let mut field_types = Vec::new();
				for f in &s.vars {
					let field_ty = self.type_to_llvm(&context, hir_mol, f.kind, &structs);
					if field_ty.is_int_type() {
						field_types.push(field_ty.into_int_type().as_basic_type_enum());
					} else if field_ty.is_float_type() {
						field_types.push(field_ty.into_float_type().as_basic_type_enum());
					} else if field_ty.is_pointer_type() {
						field_types.push(field_ty.into_pointer_type().as_basic_type_enum());
					} else if field_ty.is_struct_type() {
						field_types.push(field_ty.into_struct_type().as_basic_type_enum());
					}
				}
				structs.get(&id).unwrap().set_body(&field_types, false);
			}
		}

		let mut global_vars = HashMap::new();
		for (i, g) in hir_mol.list_global.iter().enumerate() {
			let id = crate::hir::identy::HirId::new(crate::hir::identy::HirKind::Global, 0, i as u32);
			let ty = self.type_to_llvm(&context, hir_mol, g.ty, &structs);
			let basic_ty = if ty.is_int_type() { ty.into_int_type().as_basic_type_enum() } else if ty.is_float_type() { ty.into_float_type().as_basic_type_enum() } else if ty.is_struct_type() { ty.into_struct_type().as_basic_type_enum() } else { ty.into_pointer_type().as_basic_type_enum() };
			let global_val = module.add_global(basic_ty, None, &g.name);
			global_val.set_constant(g.is_const);
			if g.is_weak {
				global_val.set_linkage(Linkage::WeakAny);
			}
			// Basic init
			global_val.set_initializer(&basic_ty.const_zero());
			global_vars.insert(id, global_val);
		}

		let mut funcs = HashMap::new();
		for (i, f) in hir_mol.list_func.iter().enumerate() {
			let id = crate::hir::identy::HirId::new(crate::hir::identy::HirKind::Func, 0, i as u32);
			let ret_ty = self.type_to_llvm(&context, hir_mol, f.ret_ty, &structs);
			let mut arg_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();
			for arg_ty_id in &f.arg_tys {
				let ty = self.type_to_llvm(&context, hir_mol, *arg_ty_id, &structs);
				let basic_ty = if ty.is_int_type() { ty.into_int_type().as_basic_type_enum() } else if ty.is_float_type() { ty.into_float_type().as_basic_type_enum() } else if ty.is_struct_type() { ty.into_struct_type().as_basic_type_enum() } else { ty.into_pointer_type().as_basic_type_enum() };
				arg_types.push(basic_ty.into());
			}
			let fn_type = if ret_ty.is_void_type() {
				ret_ty.into_void_type().fn_type(&arg_types, false)
			} else if ret_ty.is_int_type() {
				ret_ty.into_int_type().fn_type(&arg_types, false)
			} else if ret_ty.is_float_type() {
				ret_ty.into_float_type().fn_type(&arg_types, false)
			} else {
				ret_ty.into_pointer_type().fn_type(&arg_types, false)
			};
			
			let fn_val = module.add_function(&f.name, fn_type, if f.is_weak { Some(Linkage::WeakAny) } else { None });
			funcs.insert(id, fn_val);
		}

		for (i, f) in hir_mol.list_func.iter().enumerate() {
			let id = crate::hir::identy::HirId::new(crate::hir::identy::HirKind::Func, 0, i as u32);
			let fn_val = funcs.get(&id).unwrap();
			
			if f.blocks.is_empty() {
				let bb = context.append_basic_block(*fn_val, "entry");
				builder.position_at_end(bb);
				let ret_ty = self.type_to_llvm(&context, hir_mol, f.ret_ty, &structs);
				if ret_ty.is_void_type() {
					builder.build_return(None).unwrap();
				} else if ret_ty.is_int_type() {
					builder.build_return(Some(&ret_ty.into_int_type().const_zero())).unwrap();
				} else if ret_ty.is_float_type() {
					builder.build_return(Some(&ret_ty.into_float_type().const_zero())).unwrap();
				} else {
					builder.build_return(Some(&ret_ty.into_pointer_type().const_null())).unwrap();
				}
			} else {
				let mut blocks = HashMap::new();
				for block_id in &f.blocks {
					let block = hir_mol.get_block(*block_id);
					let bb = context.append_basic_block(*fn_val, &block.name);
					blocks.insert(*block_id, bb);
				}

				let mut vals: HashMap<crate::hir::identy::HirId, BasicValueEnum> = HashMap::new();

				for block_id in &f.blocks {
					let block = hir_mol.get_block(*block_id);
					builder.position_at_end(*blocks.get(block_id).unwrap());
					for instr_id in &block.instrs {
						let instr = hir_mol.get_instr(*instr_id);
						match &instr.vari {
							crate::hir::instr::HirInstrVari::Alloca(ty_id) => {
								let ty = self.type_to_llvm(&context, hir_mol, *ty_id, &structs);
								let basic_ty = if ty.is_int_type() { ty.into_int_type().as_basic_type_enum() } else if ty.is_float_type() { ty.into_float_type().as_basic_type_enum() } else if ty.is_struct_type() { ty.into_struct_type().as_basic_type_enum() } else { ty.into_pointer_type().as_basic_type_enum() };
								let ptr = builder.build_alloca(basic_ty, &format!("alloca_{}", instr_id.index())).unwrap();
								vals.insert(*instr_id, ptr.into());
							}
							crate::hir::instr::HirInstrVari::Store(val, ptr) => {
								let get_val = |v: &crate::hir::value::HirValue| -> BasicValueEnum {
									match v {
										crate::hir::value::HirValue::ConstInt(i) => context.i64_type().const_int(*i as u64, false).into(),
										crate::hir::value::HirValue::ConstFloat(f) => context.f64_type().const_float(*f).into(),
										crate::hir::value::HirValue::ConstBool(b) => context.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
										crate::hir::value::HirValue::Reg(id) => *vals.get(id).unwrap(),
										crate::hir::value::HirValue::Global(id) => global_vars.get(id).unwrap().as_pointer_value().into(),
										crate::hir::value::HirValue::Null => context.ptr_type(AddressSpace::default()).const_null().into(),
									}
								};
								let v = get_val(val);
								let p = get_val(ptr).into_pointer_value();
								builder.build_store(p, v).unwrap();
							}
							crate::hir::instr::HirInstrVari::Load(ty_id, ptr) => {
								let p = match ptr {
										crate::hir::value::HirValue::Reg(id) => *vals.get(id).unwrap(),
										crate::hir::value::HirValue::Global(id) => global_vars.get(id).unwrap().as_pointer_value().into(),
										crate::hir::value::HirValue::Null => context.ptr_type(AddressSpace::default()).const_null().into(),
										_ => panic!("Invalid pointer for load"),
								}.into_pointer_value();
								let ty = self.type_to_llvm(&context, hir_mol, *ty_id, &structs);
								let basic_ty = if ty.is_int_type() { ty.into_int_type().as_basic_type_enum() } else if ty.is_float_type() { ty.into_float_type().as_basic_type_enum() } else if ty.is_struct_type() { ty.into_struct_type().as_basic_type_enum() } else { ty.into_pointer_type().as_basic_type_enum() };
								let loaded = builder.build_load(basic_ty, p, &format!("load_{}", instr_id.index())).unwrap();
								vals.insert(*instr_id, loaded);
							}
							crate::hir::instr::HirInstrVari::Ret(opt_val) => {
								if let Some(val) = opt_val {
									let bval = match val {
										crate::hir::value::HirValue::ConstInt(i) => context.i64_type().const_int(*i as u64, false).into(),
										crate::hir::value::HirValue::ConstFloat(f) => context.f64_type().const_float(*f).into(),
										crate::hir::value::HirValue::ConstBool(b) => context.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
										crate::hir::value::HirValue::Reg(r) => *vals.get(r).unwrap(),
										crate::hir::value::HirValue::Global(r) => global_vars.get(r).unwrap().as_pointer_value().into(),
										crate::hir::value::HirValue::Null => context.ptr_type(AddressSpace::default()).const_null().into(),
									};
									builder.build_return(Some(&bval)).unwrap();
								} else {
									builder.build_return(None).unwrap();
								}
							}
							crate::hir::instr::HirInstrVari::Br(target) => {
								builder.build_unconditional_branch(*blocks.get(target).unwrap()).unwrap();
							}
							crate::hir::instr::HirInstrVari::CondBr(cond, then_block, else_block) => {
								let get_val = |v: &crate::hir::value::HirValue| -> BasicValueEnum {
									match v {
										crate::hir::value::HirValue::ConstInt(i) => context.i64_type().const_int(*i as u64, false).into(),
										crate::hir::value::HirValue::ConstFloat(f) => context.f64_type().const_float(*f).into(),
										crate::hir::value::HirValue::ConstBool(b) => context.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
										crate::hir::value::HirValue::Reg(id) => *vals.get(id).unwrap(),
										crate::hir::value::HirValue::Global(id) => global_vars.get(id).unwrap().as_pointer_value().into(),
										crate::hir::value::HirValue::Null => context.ptr_type(AddressSpace::default()).const_null().into(),
									}
								};
								let c = get_val(cond).into_int_value();
								builder.build_conditional_branch(c, *blocks.get(then_block).unwrap(), *blocks.get(else_block).unwrap()).unwrap();
							}
							crate::hir::instr::HirInstrVari::ICmp(op, lhs, rhs) => {
								let get_val = |v: &crate::hir::value::HirValue| -> BasicValueEnum {
									match v {
										crate::hir::value::HirValue::ConstInt(i) => context.i64_type().const_int(*i as u64, false).into(),
										crate::hir::value::HirValue::ConstFloat(f) => context.f64_type().const_float(*f).into(),
										crate::hir::value::HirValue::ConstBool(b) => context.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
										crate::hir::value::HirValue::Reg(id) => *vals.get(id).unwrap(),
										crate::hir::value::HirValue::Global(id) => global_vars.get(id).unwrap().as_pointer_value().into(),
										crate::hir::value::HirValue::Null => context.ptr_type(AddressSpace::default()).const_null().into(),
									}
								};
								let l = get_val(lhs).into_int_value();
								let r = get_val(rhs).into_int_value();
								let pred = match op.as_str() {
									"eq" => inkwell::IntPredicate::EQ,
									"ne" => inkwell::IntPredicate::NE,
									"slt" => inkwell::IntPredicate::SLT,
									"sgt" => inkwell::IntPredicate::SGT,
									"sle" => inkwell::IntPredicate::SLE,
									"sge" => inkwell::IntPredicate::SGE,
									_ => panic!("Unknown ICmp pred"),
								};
								let cmp = builder.build_int_compare(pred, l, r, &format!("icmp_{}", instr_id.index())).unwrap();
								vals.insert(*instr_id, cmp.into());
							}
							_ => todo!("cgen: unimplemented instr {:?}", instr.vari),
						}
					}
				}
			}
		}

		module.print_to_string().to_string()
	}
}
