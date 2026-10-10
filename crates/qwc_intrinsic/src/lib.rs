/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{CID, DefPath, Item, ItemAttr, ItemKind, ItemVis, Krate, LayoutInfo, SymVis, Type, TypeAttr, TypeKind, Visitor};
use qwc_resolve::ExportMap;
use qwc_string_interner::StrInterner;


pub fn new_core(cid: CID, sin: &mut StrInterner, layinfo: &LayoutInfo) -> (Krate, ExportMap, qwc_hir::PrimTypes) {
	let mut cre = Krate::new(cid);

	let path_root = cre.push(DefPath::Root(sin.sid_core()));
	let path_types = cre.push(DefPath::Path{base: path_root, name: sin.sid_types()});

	let mut root = vec![];

	let ty_type = cre.push(Type{kind: TypeKind::GenericSelfType, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()});
	let ty_generic_self_type = cre.push(Type{kind: TypeKind::GenericSelfType, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()});
	let ty_error = cre.push(Type{kind: TypeKind::Error, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()});
	let ty_unit = cre.push(Type{kind: TypeKind::Unit, layout: qwc_hir::Layout::new_static(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()});
	let ty_never = cre.push(Type{kind: TypeKind::Never, layout: qwc_hir::Layout::new_inhabited(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()});

	// types
	let (types, prims) = {
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

		let prims = qwc_hir::PrimTypes {
			ty_type,
			ty_generic_self_type,
			ty_error,
			ty_unit,
			ty_never,
			ty_bool,
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

	(cre, exp, prims)
}

