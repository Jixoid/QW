use qwc_arena::Files;
use qwc_ast as ast;
use qwc_diagnostic::Summary;
use qwc_hir as hir;
use qwc_resolve::{Scope, ScopeMap};
use qwc_string_interner::StrInterner;
use rustc_hash::FxHashMap;

use crate::ItemLow;


pub struct Ctx<'ast, 'hir> {
  pub src:  &'ast ast::Krate,
  pub sin:  &'ast StrInterner,
  pub far:  &'ast Files,
  pub scp:  &'ast ScopeMap,
  pub lscp: &'ast Scope,
  pub cre:  &'hir mut hir::Krate,
  pub sum:  &'hir mut Summary,
  pub cmap: &'hir mut CacheMap,
}

pub struct CacheMap {
  pub(crate) cache_type: FxHashMap<ast::TypeId, hir::TypeId>,
  pub(crate) cache_item: FxHashMap<ast::ItemId, Option<hir::ItemId>>,
  pub(crate) cache_expr: FxHashMap<ast::ExprId, hir::ExprId>,
}


#[macro_export]
macro_rules! ctx {
  ($lscp:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, sum: $ctx.sum, cmap: $ctx.cmap, src: $ctx.src, sin: $ctx.sin, far: $ctx.far, scp: $ctx.scp, $lscp}
  }
}



pub struct HGen;

impl<'ast, 'hir> HGen {

  pub fn low(src: &'ast ast::Krate, sin: &'ast StrInterner, far: &'ast Files, scp: &'ast ScopeMap) -> (Option<hir::Krate>, Summary) {
    let mut cre = hir::Krate::new();
    let mut sum = Summary::new();
    let mut cmap = CacheMap{ cache_type: FxHashMap::default(), cache_item: FxHashMap::default(), cache_expr: FxHashMap::default() };

    let root = src.root().unwrap();
    let lscp = scp.get(&root.to_any()).unwrap();

    match ItemLow::low(&mut Ctx{cre: &mut cre, sum: &mut sum, cmap: &mut cmap, src, sin, far, scp, lscp}, root) {
      Ok(root) => cre.set_root(root.unwrap()),
      
      Err(msg) => sum.add(msg),
    };

    (Some(cre), sum)
  }

}
