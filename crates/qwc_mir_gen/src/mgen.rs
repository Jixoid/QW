use qwc_diagnostic::Summary;
use qwc_hir as hir;
use qwc_mir::{self as mir, LayoutInfo};
use qwc_string_interner::{Sid, StrInterner};

use rustc_hash::FxHashMap;

use crate::{SymbLow, ty_interner::TypeInterner};


pub struct Ctx<'ast, 'hir, 'mir, 'a> {
  pub src: &'hir hir::Krate,
  pub sin: &'ast StrInterner,
  pub cre: &'mir mut mir::Krate,
  pub tin: &'mir mut TypeInterner<'a>,
  pub sum: &'mir mut Summary,
  pub mgr: &'mir mut Vec<Sid>,
  pub cmap: &'mir mut CacheMap,
}

pub struct CacheMap {
  pub(crate) cache_type: FxHashMap<hir::TypeId, mir::TypeId>,
  pub(crate) cache_item: FxHashMap<hir::ItemId, Option<mir::SymbId>>,
}


#[macro_export]
macro_rules! ctx {
  ($mgr:ident -> $ctx:expr) => {
    &mut Ctx{cre: $ctx.cre, tin: $ctx.tin, sum: $ctx.sum, src: $ctx.src, sin: $ctx.sin, cmap: $ctx.cmap, $mgr}
  };
}



pub struct MGen;

impl<'ast, 'hir, 'mir, 'a> MGen {

  pub fn low(src: &'hir hir::Krate, sin: &'ast StrInterner, layinfo: &'a LayoutInfo) -> (Option<mir::Krate>, Summary) {
    let mut cre = mir::Krate::new();
    let mut sum = Summary::new();
    let mut tin = TypeInterner::new(&mut cre, layinfo);
    let mut cmap = CacheMap {
      cache_type: FxHashMap::default(),
      cache_item: FxHashMap::default(),
    };

    let root = src.root().unwrap();

    let _ = SymbLow::low(
      &mut Ctx{cre: &mut cre, tin: &mut tin, sum: &mut sum, cmap: &mut cmap, src, sin, mgr: &mut vec![]},
      root
    ).map_err(|msg| sum.add(msg));

    (Some(cre), sum)
  }

}
