use qwc_hir::{CID, Item, ItemAttr, ItemKind, ItemVis, Krate, LayoutInfo, SymVis, Type, TypeKind, Visitor};
use qwc_resolve::ExportMap;
use qwc_string_interner::StrInterner;


pub fn new_core(cid: CID, sin: &mut StrInterner, layinfo: &LayoutInfo) -> (Krate, ExportMap) {
	let mut cre = Krate::new(cid);

	let mut root = vec![];


	// types
	let types = {
		let mut types = vec![];

		macro_rules! reg {
			($name:literal => $kind:expr, $lay:expr) => {
				let kind = cre.push(Type{kind: $kind, layout: $lay});

				let item = cre.push(Item{vis: ItemVis::Public(SymVis::Internal), kind: ItemKind::Using { kind, name: sin.sid($name) }, attr: ItemAttr::empty() });

				types.push(item);
			};
		}

		reg!("bool" => TypeKind::Bool, layinfo.bool_lay);

		reg!("i8"   => TypeKind::Int(8,  true),  layinfo.i8_lay);
		reg!("i16"  => TypeKind::Int(16, true),  layinfo.i16_lay);
		reg!("i32"  => TypeKind::Int(32, true),  layinfo.i32_lay);
		reg!("i64"  => TypeKind::Int(64, true),  layinfo.i64_lay);
		reg!("i128" => TypeKind::Int(128, true), layinfo.i128_lay);
		
		reg!("u8"   => TypeKind::Int(8,  false),  layinfo.i8_lay);
		reg!("u16"  => TypeKind::Int(16, false),  layinfo.i16_lay);
		reg!("u32"  => TypeKind::Int(32, false),  layinfo.i32_lay);
		reg!("u64"  => TypeKind::Int(64, false),  layinfo.i64_lay);
		reg!("u128" => TypeKind::Int(128, false), layinfo.i128_lay);
		
		reg!("b8"   => TypeKind::Int(8,  false),  layinfo.i8_lay);
		reg!("b16"  => TypeKind::Int(16, false),  layinfo.i16_lay);
		reg!("b32"  => TypeKind::Int(32, false),  layinfo.i32_lay);
		reg!("b64"  => TypeKind::Int(64, false),  layinfo.i64_lay);
		reg!("b128" => TypeKind::Int(128, false), layinfo.i128_lay);
		
		reg!("f16"  => TypeKind::Float(16),  layinfo.f16_lay);
		reg!("f32"  => TypeKind::Float(32),  layinfo.f32_lay);
		reg!("f64"  => TypeKind::Float(64),  layinfo.f64_lay);
		reg!("f128" => TypeKind::Float(128), layinfo.f128_lay);
		

		// Post
		let this = Item {
			vis: ItemVis::Public(SymVis::Internal),
			kind: ItemKind::NameSpace { rng: cre.extra(&types), name: sin.sid("types") },
			attr: ItemAttr::empty(),
		};

		cre.push(this)
	};
	root.push(types);


	// root
	let root = {
		let this = Item {
			vis: ItemVis::Public(SymVis::Internal),
			kind: ItemKind::RootNS{ rng: cre.extra(&root) },
			attr: ItemAttr::empty(),
		};

		cre.push(this)
	};

	cre.set_root(root);


	let exp = ExportMap::visit(&cre).map_err(|_| panic!()).unwrap();

	(cre, exp)
}
