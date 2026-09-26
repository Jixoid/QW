use qwc_hir::{CID, Item, ItemKind, ItemVis, Krate, LayoutInfo, Type, TypeKind, Visitor};
use qwc_resolve::ExportMap;
use qwc_string_interner::StrInterner;


pub fn new_core(cid: CID, sin: &mut StrInterner, layinfo: &LayoutInfo) -> (Krate, ExportMap) {
	let mut cre = Krate::new(cid);

	let mut root = vec![];


	// types
	let types = {
		let mut types = vec![];


		let bool_ty = cre.push(Type{
			kind: TypeKind::Bool,
			layout: layinfo.bool_lay,
		});

		let bool_it = Item {
			vis: ItemVis::Public,
			svis: None,
			kind: ItemKind::Using { kind: bool_ty, name: sin.sid("bool") }
		};
		types.push(cre.push(bool_it));


		// Post
		let this = Item {
			vis: ItemVis::Public,
			svis: None,
			kind: ItemKind::NameSpace { rng: cre.extra(&types), name: sin.sid("types") }
		};

		cre.push(this)
	};
	root.push(types);


	// root
	let root = {
		let this = Item {
			vis: ItemVis::Public,
			svis: None,
			kind: ItemKind::RootNS{ rng: cre.extra(&root) }
		};

		cre.push(this)
	};

	cre.set_root(root);


	let exp = ExportMap::visit(&cre).map_err(|_| panic!()).unwrap();

	(cre, exp)
}
