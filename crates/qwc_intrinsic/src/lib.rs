/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{Rng, CID, DefPath, Item, ItemAttr, ItemKind, ItemVis, Krate, Layout, LayoutInfo, PrimOps, PrimTypes, SymVis, Thing, Type, TypeAttr, TypeKind};
use qwc_resolve::ExportMap;
use qwc_string_interner::StrInterner;


pub fn new_core(cid: CID, sin: &mut StrInterner, layinfo: &LayoutInfo) -> (Krate, ExportMap, PrimTypes, PrimOps) {
	let mut cre = Krate::new(cid);

	let path_root = cre.push(DefPath::Root(sin.sid_core()));
	let path_types = cre.push(DefPath::Path{base: path_root, name: sin.sid_types()});

	let mut root = vec![];

	let ty_type = cre.push(Type{kind: TypeKind::GenericSelfType, layout: qwc_hir::Layout::STATIC, attr: TypeAttr::empty()});
	let ty_generic_self_type = cre.push(Type{kind: TypeKind::GenericSelfType, layout: qwc_hir::Layout::STATIC, attr: TypeAttr::empty()});
	let ty_generic_ref_mut_self_type = cre.push(Type{kind: TypeKind::Ref{kind: ty_generic_self_type, ism: true, addrspace: None}, layout: layinfo.ptr_size, attr: TypeAttr::empty()});
	let ty_generic_ref_self_type = cre.push(Type{kind: TypeKind::Ref{kind: ty_generic_self_type, ism: false, addrspace: None}, layout: layinfo.ptr_size, attr: TypeAttr::empty()});
	let ty_error = cre.push(Type{kind: TypeKind::Error, layout: qwc_hir::Layout::STATIC, attr: TypeAttr::empty()});
	let ty_unit = cre.push(Type{kind: TypeKind::Unit, layout: qwc_hir::Layout::STATIC, attr: TypeAttr::empty()});
	let ty_never = cre.push(Type{kind: TypeKind::Never, layout: qwc_hir::Layout::new_inhabited(), attr: TypeAttr::empty()});

	// types
	let (types, prim_ty) = {
		let mut types = vec![];

		macro_rules! reg {
			($name:literal => $kind:expr, $lay:expr) => {{
				let name_sid = sin.sid($name);
				let path = cre.push(DefPath::Path{base: path_types, name: name_sid});
				let kind = cre.push(Type{kind: $kind, layout: $lay, attr: TypeAttr::empty()});

				let item = cre.push(Item{
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(name_sid),
					kind: ItemKind::Using{kind},
					path,
					attr: ItemAttr::empty()
				});

				types.push(item);
				kind
			}};
		}

		let ty_bool = reg!("bool" => TypeKind::Bool, layinfo.bool_lay);
		let ty_char = reg!("char" => TypeKind::Char, layinfo.bool_lay);
		
		let ty_str = reg!("str" => TypeKind::Str, layinfo.str_lay);

		let ty_isize = reg!("isize" => TypeKind::ArchInt(true), layinfo.ptr_size);
		let ty_usize = reg!("usize" => TypeKind::ArchInt(false), layinfo.ptr_size);

		let ty_i8   = reg!("i8"   => TypeKind::Int(8,  true),  layinfo.i8_lay);
		let ty_i16  = reg!("i16"  => TypeKind::Int(16, true),  layinfo.i16_lay);
		let ty_i32  = reg!("i32"  => TypeKind::Int(32, true),  layinfo.i32_lay);
		let ty_i64  = reg!("i64"  => TypeKind::Int(64, true),  layinfo.i64_lay);
		let ty_i128 = reg!("i128" => TypeKind::Int(128, true), layinfo.i128_lay);
		
		let ty_u8   = reg!("u8"   => TypeKind::Int(8,  false),  layinfo.i8_lay);
		let ty_u16  = reg!("u16"  => TypeKind::Int(16, false),  layinfo.i16_lay);
		let ty_u32  = reg!("u32"  => TypeKind::Int(32, false),  layinfo.i32_lay);
		let ty_u64  = reg!("u64"  => TypeKind::Int(64, false),  layinfo.i64_lay);
		let ty_u128 = reg!("u128" => TypeKind::Int(128, false), layinfo.i128_lay);
		
		let ty_b8   = reg!("b8"   => TypeKind::Bit(8),   layinfo.i8_lay);
		let ty_b16  = reg!("b16"  => TypeKind::Bit(16),  layinfo.i16_lay);
		let ty_b32  = reg!("b32"  => TypeKind::Bit(32),  layinfo.i32_lay);
		let ty_b64  = reg!("b64"  => TypeKind::Bit(64),  layinfo.i64_lay);
		let ty_b128 = reg!("b128" => TypeKind::Bit(128), layinfo.i128_lay);
		
		let ty_f16  = reg!("f16"  => TypeKind::Float(16),  layinfo.f16_lay);
		let ty_f32  = reg!("f32"  => TypeKind::Float(32),  layinfo.f32_lay);
		let ty_f64  = reg!("f64"  => TypeKind::Float(64),  layinfo.f64_lay);
		let ty_f128 = reg!("f128" => TypeKind::Float(128), layinfo.f128_lay);

		let prims = PrimTypes {
			ty_type,
			ty_generic_self_type,
			ty_error,
			ty_unit,
			ty_never,
			ty_bool,
			ty_char,
			ty_str,
			ty_isize,
			ty_usize,
			ty_i8,
			ty_i16,
			ty_i32,
			ty_i64,
			ty_i128,
			ty_u8,
			ty_u16,
			ty_u32,
			ty_u64,
			ty_u128,
			ty_b8,
			ty_b16,
			ty_b32,
			ty_b64,
			ty_b128,
			ty_f16,
			ty_f32,
			ty_f64,
			ty_f128,
		};

		// Post
		let this = Item {
			vis: ItemVis::Public(SymVis::Internal),
			name: Some(sin.sid_types()),
			kind: ItemKind::NameSpace { rng: cre.extra(&types) },
			path: path_types,
			attr: ItemAttr::empty(),
		};

		(cre.push(this), prims)
	};
	root.push(types);

	// ops
	let (ops, prim_op) = {
		let path_ops = cre.push(DefPath::Path{base: path_root, name: sin.sid("ops")});

		macro_rules! def_bin_op {
			($its:expr, $trait_name:literal, $method_name:literal) => {{
				let sid_trait = sin.sid($trait_name);
				let sid_method = sin.sid($method_name);
				let sid_rhs = sin.sid("Rhs");
				let sid_output = sin.sid("Output");
				let sid_arg_rhs = sin.sid("rhs");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// generic<Rhs>
				let param_rhs = cre.push(Thing::NamedType(sid_rhs, ty_type));
				let generic_args = cre.extra(&[param_rhs]);

				let empty_items: &[qwc_hir::ItemId] = &[];
				let empty_rng = cre.extra(empty_items);
				let generic_item = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: empty_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// GenericType for Rhs: generic_item, idx 0
				let ty_rhs = cre.push(Type {
					kind: TypeKind::GenericType { generic: generic_item, idx: 0 },
					layout: Layout::STATIC,
					attr: TypeAttr::empty(),
				});

				// AssociatedType for Output: trait_ty, idx 0
				let ty_output = cre.push(Type {
					kind: TypeKind::AssociatedType { trait_ty, idx: 0 },
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// fun $method(self, rhs: Rhs) -> Output
				let thing_rhs = cre.push(Thing::NamedType(sid_arg_rhs, ty_rhs));
				let fn_args = cre.extra(&[thing_rhs]);

				let ty_fn = cre.push(Type {
					kind: TypeKind::Fun {
						self_kind: Some(ty_generic_self_type),
						args: fn_args,
						ret: ty_output,
					},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				let thing_method = cre.push(Thing::NamedType(sid_method, ty_fn));
				let methods = cre.extra(&[thing_method]);

				let thing_type_output = cre.push(Thing::Name(sid_output));
				let types = cre.extra(&[thing_type_output]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait{methods, types, reqs: Rng::empty(cid), path: path_trait},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::META,
					attr: TypeAttr::empty(),
				});

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: ty_generic_raw },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				let item_trait_rng = cre.extra(&[item_trait]);
				*cre.get_mut(generic_item) = Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: item_trait_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				};

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		macro_rules! def_bin_assign_op {
			($its:expr, $trait_name:literal, $method_name:literal) => {{
				let sid_trait = sin.sid($trait_name);
				let sid_method = sin.sid($method_name);
				let sid_rhs = sin.sid("Rhs");
				let sid_arg_rhs = sin.sid("rhs");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// generic<Rhs>
				let param_rhs = cre.push(Thing::NamedType(sid_rhs, ty_type));
				let generic_args = cre.extra(&[param_rhs]);

				let empty_items: &[qwc_hir::ItemId] = &[];
				let empty_rng = cre.extra(empty_items);
				let generic_item = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: empty_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// GenericType for Rhs: generic_item, idx 0
				let ty_rhs = cre.push(Type {
					kind: TypeKind::GenericType { generic: generic_item, idx: 0 },
					layout: Layout::STATIC,
					attr: TypeAttr::empty(),
				});

				// fun $method(&mut self, rhs: Rhs) -> ()
				let thing_rhs = cre.push(Thing::NamedType(sid_arg_rhs, ty_rhs));
				let fn_args = cre.extra(&[thing_rhs]);

				let ty_fn = cre.push(Type {
					kind: TypeKind::Fun {
						self_kind: Some(ty_generic_ref_mut_self_type),
						args: fn_args,
						ret: ty_unit,
					},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				let thing_method = cre.push(Thing::NamedType(sid_method, ty_fn));
				let methods = cre.extra(&[thing_method]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait{methods, types: Rng::empty(cid), reqs: Rng::empty(cid), path: path_trait},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::META,
					attr: TypeAttr::empty(),
				});

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: ty_generic_raw },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				let item_trait_rng = cre.extra(&[item_trait]);
				*cre.get_mut(generic_item) = Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: item_trait_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				};

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		let mut its = vec![];
		
		let op_add = def_bin_op!(its, "Add", "add");
		let op_sub = def_bin_op!(its, "Sub", "sub");
		let op_mul = def_bin_op!(its, "Mul", "mul");
		let op_div = def_bin_op!(its, "Div", "div");
		let op_rem = def_bin_op!(its, "Rem", "rem");

		macro_rules! def_unary_op {
			($its:expr, $trait_name:literal, $method_name:literal) => {{
				let sid_trait = sin.sid($trait_name);
				let sid_method = sin.sid($method_name);
				let sid_output = sin.sid("Output");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// AssociatedType for Output: trait_ty, idx 0
				let ty_output = cre.push(Type {
					kind: TypeKind::AssociatedType { trait_ty, idx: 0 },
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// fun $method(self) -> Output
				let empty_args: &[qwc_hir::ThingId] = &[];
				let fn_args = cre.extra(empty_args);

				let ty_fn = cre.push(Type {
					kind: TypeKind::Fun {
						self_kind: Some(ty_generic_self_type),
						args: fn_args,
						ret: ty_output,
					},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				let thing_method = cre.push(Thing::NamedType(sid_method, ty_fn));
				let methods = cre.extra(&[thing_method]);

				let thing_type_output = cre.push(Thing::Name(sid_output));
				let types = cre.extra(&[thing_type_output]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait{methods, types, reqs: Rng::empty(cid), path: path_trait},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: trait_ty },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		macro_rules! def_partial_eq {
			($its:expr) => {{
				let sid_trait = sin.sid("PartialEq");
				let sid_method_eq = sin.sid("eq");
				let sid_method_ne = sin.sid("ne");
				let sid_rhs = sin.sid("Rhs");
				let sid_arg_other = sin.sid("other");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// generic<Rhs>
				let param_rhs = cre.push(Thing::NamedType(sid_rhs, ty_type));
				let generic_args = cre.extra(&[param_rhs]);

				let empty_items: &[qwc_hir::ItemId] = &[];
				let empty_rng = cre.extra(empty_items);
				let generic_item = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: empty_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// GenericType for Rhs: generic_item, idx 0
				let ty_rhs = cre.push(Type {
					kind: TypeKind::GenericType { generic: generic_item, idx: 0 },
					layout: Layout::STATIC,
					attr: TypeAttr::empty(),
				});

				// &Rhs
				let ty_ref_rhs = cre.push(Type {
					kind: TypeKind::Ref{kind: ty_rhs, ism: false, addrspace: None},
					layout: layinfo.ptr_size,
					attr: TypeAttr::empty(),
				});

				// fun eq(&self, other: &Rhs) -> bool
				let thing_other = cre.push(Thing::NamedType(sid_arg_other, ty_ref_rhs));
				let fn_args = cre.extra(&[thing_other]);

				let ty_fn_eq = cre.push(Type {
					kind: TypeKind::Fun {
						self_kind: Some(ty_generic_ref_self_type),
						args: fn_args,
						ret: prim_ty.ty_bool,
					},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// fun ne(&self, other: &Rhs) -> bool
				let thing_other_ne = cre.push(Thing::NamedType(sid_arg_other, ty_ref_rhs));
				let fn_args_ne = cre.extra(&[thing_other_ne]);

				let ty_fn_ne = cre.push(Type {
					kind: TypeKind::Fun {
						self_kind: Some(ty_generic_ref_self_type),
						args: fn_args_ne,
						ret: prim_ty.ty_bool,
					},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				let thing_method_eq = cre.push(Thing::NamedType(sid_method_eq, ty_fn_eq));
				let thing_method_ne = cre.push(Thing::NamedType(sid_method_ne, ty_fn_ne));
				let methods = cre.extra(&[thing_method_eq, thing_method_ne]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait{methods, types: Rng::empty(cid), reqs: Rng::empty(cid), path: path_trait},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::META,
					attr: TypeAttr::empty(),
				});

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: ty_generic_raw },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				let item_trait_rng = cre.extra(&[item_trait]);
				*cre.get_mut(generic_item) = Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: item_trait_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				};

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		macro_rules! def_eq {
			($its:expr) => {{
				let sid_trait = sin.sid("Eq");
				let sid_rhs = sin.sid("Rhs");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// generic<Rhs>
				let param_rhs = cre.push(Thing::NamedType(sid_rhs, ty_type));
				let generic_args = cre.extra(&[param_rhs]);

				let empty_items: &[qwc_hir::ItemId] = &[];
				let empty_rng = cre.extra(empty_items);
				let generic_item = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: empty_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait{methods: Rng::empty(cid), types: Rng::empty(cid), reqs: Rng::empty(cid), path: path_trait},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::META,
					attr: TypeAttr::empty(),
				});

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: ty_generic_raw },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				let item_trait_rng = cre.extra(&[item_trait]);
				*cre.get_mut(generic_item) = Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: item_trait_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				};

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		let (ty_ordering, item_ordering) = {
			let sid_ordering = sin.sid_ordering();
			let sid_less = sin.sid_less();
			let sid_equal = sin.sid_equal();
			let sid_greater = sin.sid_greater();

			let path_ordering = cre.push(DefPath::Path { base: path_ops, name: sid_ordering });

			let thing_less = cre.push(Thing::NamedConst(sid_less, qwc_hir::Const::I32(-1)));
			let thing_equal = cre.push(Thing::NamedConst(sid_equal, qwc_hir::Const::I32(0)));
			let thing_greater = cre.push(Thing::NamedConst(sid_greater, qwc_hir::Const::I32(1)));
			let enum_fields = cre.extra(&[thing_less, thing_equal, thing_greater]);

			let ty_ordering = cre.push(Type {
				kind: TypeKind::ItemEnum { values: enum_fields, path: path_ordering },
				layout: layinfo.i32_lay,
				attr: TypeAttr::empty(),
			});

			let item_ordering = cre.push(Item {
				vis: ItemVis::Public(SymVis::Internal),
				name: Some(sid_ordering),
				kind: ItemKind::Using { kind: ty_ordering },
				path: path_ordering,
				attr: ItemAttr::empty(),
			});

			(ty_ordering, item_ordering)
		};
		its.push(item_ordering);

		let ty_opt_ordering = cre.push(Type {
			kind: TypeKind::Option(ty_ordering),
			layout: layinfo.i32_lay,
			attr: TypeAttr::empty(),
		});

		macro_rules! def_partial_ord {
			($its:expr, $op_peq:expr, $ty_opt_ord:expr) => {{
				let sid_trait = sin.sid_partial_ord();
				let sid_method_pc = sin.sid_partial_cmp();
				let sid_method_lt = sin.sid_lt();
				let sid_method_le = sin.sid_le();
				let sid_method_gt = sin.sid_gt();
				let sid_method_ge = sin.sid_ge();
				let sid_rhs = sin.sid("Rhs");
				let sid_arg_other = sin.sid("other");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// generic<Rhs>
				let param_rhs = cre.push(Thing::NamedType(sid_rhs, ty_type));
				let generic_args = cre.extra(&[param_rhs]);

				let empty_items: &[qwc_hir::ItemId] = &[];
				let empty_rng = cre.extra(empty_items);
				let generic_item = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: empty_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// GenericType for Rhs: generic_item, idx 0
				let ty_rhs = cre.push(Type {
					kind: TypeKind::GenericType { generic: generic_item, idx: 0 },
					layout: Layout::STATIC,
					attr: TypeAttr::empty(),
				});

				// &Rhs
				let ty_ref_rhs = cre.push(Type {
					kind: TypeKind::Ref{kind: ty_rhs, ism: false, addrspace: None},
					layout: layinfo.ptr_size,
					attr: TypeAttr::empty(),
				});

				macro_rules! make_method {
					($ret_ty:expr) => {{
						let thing_other = cre.push(Thing::NamedType(sid_arg_other, ty_ref_rhs));
						let fn_args = cre.extra(&[thing_other]);
						cre.push(Type {
							kind: TypeKind::Fun {
								self_kind: Some(ty_generic_ref_self_type),
								args: fn_args,
								ret: $ret_ty,
							},
							layout: Layout::DSAT,
							attr: TypeAttr::empty(),
						})
					}};
				}

				let ty_fn_pc = make_method!($ty_opt_ord);
				let ty_fn_lt = make_method!(prim_ty.ty_bool);
				let ty_fn_le = make_method!(prim_ty.ty_bool);
				let ty_fn_gt = make_method!(prim_ty.ty_bool);
				let ty_fn_ge = make_method!(prim_ty.ty_bool);

				let thing_pc = cre.push(Thing::NamedType(sid_method_pc, ty_fn_pc));
				let thing_lt = cre.push(Thing::NamedType(sid_method_lt, ty_fn_lt));
				let thing_le = cre.push(Thing::NamedType(sid_method_le, ty_fn_le));
				let thing_gt = cre.push(Thing::NamedType(sid_method_gt, ty_fn_gt));
				let thing_ge = cre.push(Thing::NamedType(sid_method_ge, ty_fn_ge));
				let methods = cre.extra(&[thing_pc, thing_lt, thing_le, thing_gt, thing_ge]);

				let reqs = cre.extra(&[$op_peq]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait { methods, types: Rng::empty(cid), reqs, path: path_trait },
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::META,
					attr: TypeAttr::empty(),
				});

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: ty_generic_raw },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				let item_trait_rng = cre.extra(&[item_trait]);
				*cre.get_mut(generic_item) = Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: item_trait_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				};

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		macro_rules! def_ord {
			($its:expr, $op_pord:expr, $op_eq:expr, $ty_ord:expr) => {{
				let sid_trait = sin.sid_ord();
				let sid_method_cmp = sin.sid_cmp();
				let sid_rhs = sin.sid("Rhs");
				let sid_arg_other = sin.sid("other");

				let path_trait = cre.push(DefPath::Path { base: path_ops, name: sid_trait });

				// generic<Rhs>
				let param_rhs = cre.push(Thing::NamedType(sid_rhs, ty_type));
				let generic_args = cre.extra(&[param_rhs]);

				let empty_items: &[qwc_hir::ItemId] = &[];
				let empty_rng = cre.extra(empty_items);
				let generic_item = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: empty_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				// Pre-allocate trait_ty
				let trait_ty = cre.push(Type {
					kind: TypeKind::Error,
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				// fun cmp(&self, other: &Self) -> Ordering
				let thing_other = cre.push(Thing::NamedType(sid_arg_other, ty_generic_ref_self_type));
				let fn_args = cre.extra(&[thing_other]);

				let ty_fn_cmp = cre.push(Type {
					kind: TypeKind::Fun {
						self_kind: Some(ty_generic_ref_self_type),
						args: fn_args,
						ret: $ty_ord,
					},
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				});

				let thing_cmp = cre.push(Thing::NamedType(sid_method_cmp, ty_fn_cmp));
				let methods = cre.extra(&[thing_cmp]);

				let reqs = cre.extra(&[$op_pord, $op_eq]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::ItemTrait { methods, types: Rng::empty(cid), reqs, path: path_trait },
					layout: Layout::DSAT,
					attr: TypeAttr::empty(),
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::META,
					attr: TypeAttr::empty(),
				});

				let item_trait = cre.push(Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: Some(sid_trait),
					kind: ItemKind::Using { kind: ty_generic_raw },
					path: path_trait,
					attr: ItemAttr::empty(),
				});

				let item_trait_rng = cre.extra(&[item_trait]);
				*cre.get_mut(generic_item) = Item {
					vis: ItemVis::Public(SymVis::Internal),
					name: None,
					kind: ItemKind::GenericNS { rng: item_trait_rng, args: generic_args },
					path: path_trait,
					attr: ItemAttr::empty(),
				};

				$its.push(item_trait);
				
				trait_ty
			}};
		}

		let op_add_assign = def_bin_assign_op!(its, "AddAssign", "add_assign");
		let op_sub_assign = def_bin_assign_op!(its, "SubAssign", "sub_assign");
		let op_mul_assign = def_bin_assign_op!(its, "MulAssign", "mul_assign");
		let op_div_assign = def_bin_assign_op!(its, "DivAssign", "div_assign");
		let op_rem_assign = def_bin_assign_op!(its, "RemAssign", "rem_assign");

		let op_neg = def_unary_op!(its, "Neg", "neg");
		let op_not = def_unary_op!(its, "Not", "not");
		let op_deref = def_unary_op!(its, "Deref", "deref");

		let op_partial_eq = def_partial_eq!(its);
		let op_eq = def_eq!(its);
		let op_partial_ord = def_partial_ord!(its, op_partial_eq, ty_opt_ordering);
		let op_ord = def_ord!(its, op_partial_ord, op_eq, ty_ordering);

		let prim_op = PrimOps {
			op_add,
			op_sub,
			op_mul,
			op_div,
			op_rem,
			op_add_assign,
			op_sub_assign,
			op_mul_assign,
			op_div_assign,
			op_rem_assign,
			op_neg,
			op_not,
			op_deref,
			op_partial_eq,
			op_eq,
			op_partial_ord,
			op_ord,
			ty_ordering,
		};

		let ops_items_rng = cre.extra(&its);
		let this = Item {
			vis: ItemVis::Public(SymVis::Internal),
			name: Some(sin.sid("ops")),
			kind: ItemKind::NameSpace { rng: ops_items_rng },
			path: path_ops,
			attr: ItemAttr::empty(),
		};

		(cre.push(this), prim_op)
	};
	root.push(ops);


	// root
	let root = {
		let this = Item {
			vis: ItemVis::Public(SymVis::Internal),
			name: None,
			kind: ItemKind::RootNS{ rng: cre.extra(&root) },
			path: path_root,
			attr: ItemAttr::empty(),
		};

		cre.push(this)
	};

	cre.set_root(root);


	let deps = qwc_hir::Deps::default();
	let exp = ExportMap::visit(&cre, &deps, sin).map_err(|_| ()).unwrap();

	(cre, exp, prim_ty, prim_op)
}

