use qwc_arena::Files;
use qwc_ast as ast;
use qwc_diagnostic::Summary;
use qwc_hir as hir;
use qwc_resolve::{Scope, ScopeMap};
use qwc_string_interner::StrInterner;

use crate::ItemLow;


pub struct Ctx<'ast, 'hir> {
  pub src:  &'ast ast::Krate,
  pub sin:  &'ast StrInterner,
  pub far:  &'ast Files,
  pub scp:  &'ast ScopeMap,
  pub lscp: &'ast Scope,
  pub cre:  &'hir mut hir::Krate,
  pub sum:  &'hir mut Summary,
}

#[macro_export]
macro_rules! ctx {
  ($ctx:expr => $cre:ident, $sum:ident, $src:ident, $sin:ident, $far:ident, $scp:ident, $lscp:ident) => {
    #[allow(unused_variables)]
    let Ctx{$cre, $sum, $src, $sin, $far, $scp, $lscp} = $ctx;
  };
  
  ($cre:ident, $sum:ident, $src:ident, $sin:ident, $far:ident, $scp:ident, $lscp:ident) => {
    &mut Ctx{$cre, $sum, $src, $sin, $far, $scp, $lscp}
  };
}



pub struct HGen;

impl<'ast, 'hir> HGen {

  pub fn low(src: &'ast ast::Krate, sin: &'ast StrInterner, far: &'ast Files, scp: &'ast ScopeMap) -> (Option<hir::Krate>, Summary) {
    let mut cre = hir::Krate::new();
    let mut sum = Summary::new();

    let root = src.root().unwrap();
    let lscp = scp.get(&root.to_any()).unwrap();

    match ItemLow::low(&mut Ctx{cre: &mut cre, sum: &mut sum, src, sin, far, scp, lscp}, root) {
      Ok(root) => cre.set_root(root.unwrap()),
      
      Err(msg) => sum.add(msg),
    };

    (Some(cre), sum)
  }

}
