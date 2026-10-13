/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{CID, DefPath, Item, ItemAttr, ItemKind, ItemVis, Krate, Layout, LayoutBy, LayoutInfo, PrimOps, PrimTypes, SymVis, Thing, Type, TypeAttr, TypeKind};
use qwc_resolve::ExportMap;
use qwc_string_interner::StrInterner;


pub fn new_core(cid: CID, sin: &mut StrInterner, layinfo: &LayoutInfo) -> (Krate, ExportMap, PrimTypes, PrimOps) {
	let mut cre = Krate::new(cid);

	let path_root = cre.push(DefPath::Root(sin.sid_core()));
	let path_types = cre.push(DefPath::Path{base: path_root, name: sin.sid_types()});

	let mut root = vec![];

	let path_type = cre.push(DefPath::Path{base: path_types, name: sin.sid("type")});
	let path_self = cre.push(DefPath::Path{base: path_types, name: sin.sid("Self")});
	let path_error = cre.push(DefPath::Path{base: path_types, name: sin.sid("{error}")});
	let path_unit = cre.push(DefPath::Path{base: path_types, name: sin.sid("()")});
	let path_never = cre.push(DefPath::Path{base: path_types, name: sin.sid("!")});

	let ty_type = cre.push(Type{kind: TypeKind::GenericSelfType, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty(), path: path_type});
	let ty_generic_self_type = cre.push(Type{kind: TypeKind::GenericSelfType, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty(), path: path_self});
	let ty_error = cre.push(Type{kind: TypeKind::Error, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty(), path: path_error});
	let ty_unit = cre.push(Type{kind: TypeKind::Unit, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty(), path: path_unit});
	let ty_never = cre.push(Type{kind: TypeKind::Never, layout: qwc_hir::Layout::new_inhabited(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty(), path: path_never});

	// types
	let (types, prim_ty) = {
		let mut types = vec![];

		macro_rules! reg {
			($name:literal => $kind:expr, $lay:expr) => {{
				let name_sid = sin.sid($name);
				let path = cre.push(DefPath::Path{base: path_types, name: name_sid});
				let kind = cre.push(Type{kind: $kind, layout: $lay, attr: TypeAttr::empty(), path});

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
		
		let ty_b8   = reg!("b8"   => TypeKind::Int(8,  false),  layinfo.i8_lay);
		let ty_b16  = reg!("b16"  => TypeKind::Int(16, false),  layinfo.i16_lay);
		let ty_b32  = reg!("b32"  => TypeKind::Int(32, false),  layinfo.i32_lay);
		let ty_b64  = reg!("b64"  => TypeKind::Int(64, false),  layinfo.i64_lay);
		let ty_b128 = reg!("b128" => TypeKind::Int(128, false), layinfo.i128_lay);
		
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
				let path_fn = cre.push(DefPath::Path { base: path_trait, name: sid_method });
				let path_rhs = cre.push(DefPath::Path { base: path_trait, name: sid_rhs });
				let path_output = cre.push(DefPath::Path { base: path_trait, name: sid_output });

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
					layout: Layout::new_dsat(LayoutBy::QW),
					attr: TypeAttr::empty(),
					path: path_trait,
				});

				// GenericType for Rhs: generic_item, idx 0
				let ty_rhs = cre.push(Type {
					kind: TypeKind::GenericType { generic: generic_item, idx: 0 },
					layout: Layout::new_static(LayoutBy::QW),
					attr: TypeAttr::empty(),
					path: path_rhs,
				});

				// AssociatedType for Output: trait_ty, idx 0
				let ty_output = cre.push(Type {
					kind: TypeKind::AssociatedType { trait_ty, idx: 0 },
					layout: Layout::new_dsat(LayoutBy::QW),
					attr: TypeAttr::empty(),
					path: path_output,
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
					layout: Layout::new_dsat(LayoutBy::QW),
					attr: TypeAttr::empty(),
					path: path_fn,
				});

				let thing_method = cre.push(Thing::NamedType(sid_method, ty_fn));
				let methods = cre.extra(&[thing_method]);

				let thing_type_output = cre.push(Thing::Name(sid_output));
				let types = cre.extra(&[thing_type_output]);

				*cre.get_mut(trait_ty) = Type {
					kind: TypeKind::Trait(methods, types),
					layout: Layout::new_dsat(LayoutBy::QW),
					attr: TypeAttr::empty(),
					path: path_trait,
				};

				let ty_generic_raw = cre.push(Type {
					kind: TypeKind::GenericRaw { generic: generic_item, kind: trait_ty },
					layout: Layout::new_meta(LayoutBy::QW),
					attr: TypeAttr::empty(),
					path: path_trait,
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

		let prim_op = PrimOps {
			op_add,
			op_sub,
			op_mul,
			op_div,
			op_rem,
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


	let exp = ExportMap::visit(&cre).map_err(|_| panic!()).unwrap();

	(cre, exp, prim_ty, prim_op)
}

