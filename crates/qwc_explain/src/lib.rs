/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;
use owo_colors::OwoColorize;

use qwc_arena::Files;
use qwc_ast::{self as ast, PushOkApi};
use qwc_hir::self as hir;
use qwc_diagnostic::msg::*;
use qwc_hir_gen::HGen;
use qwc_parse::Parse;
use qwc_resolve::{Imod, ScopeCollector};
use qwc_string_interner::StrInterner;


fn test_code(file: &str) {
	let ast = &mut ast::Krate::default();
  let sin = &mut StrInterner::default();
  let sid = sin.sid("test");
  let far = &mut Files::default();
  let fi = {
    let fid = far.add_buffer(Path::new("test.qw"), file.to_string().into_bytes());
    far.get(fid)
  };


	// Parse
	let root_path = ast.push(ast::DefPath::Root(sid));
	let (pos, rng, sum) = Parse::parse(ast, root_path, sin, far, fi);
	let root = ast::Item {
    pos,
    vis: ast::Visibility::Inherited,
    name: None,
    kind: ast::ItemKind::Krate(rng),
    path: root_path,
  }.push(ast);

	ast.set_root(root);


	// Write
	println!();
	println!("{}", ast::Dump{cre: ast, far, sin});
	println!();

  if !sum.is_empty() {
    for msg in &sum {
      println!("{}", msg.display_human(far));
    }
    println!("{}", sum);

    return;
  }


	// Layout
  let hir_layinfo = hir::LayoutInfo{
    bool_lay: hir::Layout::STATIC,
    
    str_lay: hir::Layout::DST,

    i8_lay:   hir::Layout::STATIC,
    i16_lay:  hir::Layout::STATIC,
    i32_lay:  hir::Layout::STATIC,
    i64_lay:  hir::Layout::STATIC,
    i128_lay: hir::Layout::STATIC,

    bf16_lay: hir::Layout::STATIC,
    f16_lay:  hir::Layout::STATIC,
    f32_lay:  hir::Layout::STATIC,
    f64_lay:  hir::Layout::STATIC,
    f128_lay: hir::Layout::STATIC,

    ptr_size: hir::Layout::STATIC,
  };


	// Core & Imods
  let mut deps = hir::Deps::default();
  let core_cid = deps.get_next_id();
  let (core_cre, core_exp, prims, prim_op) = qwc_intrinsic::new_core(core_cid, sin, &hir_layinfo);
  deps.add(core_cre);
  deps.set_prims(prims);
  deps.set_prim_op(prim_op);
  let core_name = sin.sid("core");
  let imods = [Imod::new(core_name, &core_exp)];
  let imod_cids = [core_cid];


	// Scope
	let (scp, implst) = ScopeCollector::collect(ast, sin, &imods).map_err(|_| ()).unwrap();


	// HGen
	let (_hir, sum) = HGen::low(ast, sin, far, &scp, implst, &imod_cids, &mut deps);

	if !sum.is_empty() {
    for msg in &sum {
      println!("{}", msg.display_human(far));
    }
    println!("{}", sum);
  }
}


pub fn explain(code: String) {
	let code = match u16::from_str_radix(&code, 16) {
		Ok(v) => v,
		Err(_) => {
			eprintln!("{}{} E{} is not a valid error code", "error".red().bold(), ":".bright_black(), code);
			return;
		}
	};


	if MUTUALLY_CONTRADICTORY_DEFINITIONS.code().unwrap() == code {
		println!("This error indicates that the specified properties are incompatible with each other.");
		
		test_code(r#"
  ![import, export]
  fun test() {}
"#);
	}
	
	else if MISMATCHED_TYPES.code().unwrap() == code {
		println!("This error occurs when a different type is assigned or given than the expected type.");
		
		test_code(r#"
  use core::types::*;

  ![entry]
  pub fun main() {
    let a: i32 = 1.0;
  }
"#);
	}

	else {
		eprintln!("{}{} E{:04x} is not a valid error code", "error".red().bold(), ":".bright_black(), code)
	}
}
