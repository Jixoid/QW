/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Summary;
use qwc_hir as hir;
use qwc_mir::{self as mir, LayoutInfo};
use qwc_string_interner::StrInterner;

use rustc_hash::FxHashMap;

use crate::{SymbLow, ty_interner::TypeInterner};


pub struct Ctx<'ast, 'hir, 'mir, 'a> {
  pub src: &'hir hir::Krate,
  pub deps: &'hir hir::Deps,
  pub sin: &'ast StrInterner,
  pub cre: &'mir mut mir::Krate,
  pub tin: &'mir mut TypeInterner<'a>,
  pub cmap: &'mir mut CacheMap,
}

impl<'ast, 'hir, 'mir, 'a> Ctx<'ast, 'hir, 'mir, 'a> {
  pub fn get_krate(&self, cid: hir::CID) -> &hir::Krate {
    if cid == self.src.cid() {
      self.src
    } else {
      self.deps.get(cid)
    }
  }

  pub fn get_type(&self, id: hir::TypeId) -> &hir::Type {
    self.get_krate(id.cid()).get(id)
  }
}

pub struct CacheMap {
  pub(crate) cache_type: FxHashMap<hir::TypeId, mir::TypeId>,
  pub(crate) cache_item: FxHashMap<hir::ItemId, Option<mir::SymbId>>,
  pub(crate) cache_vmt: FxHashMap<(hir::TypeId, hir::TypeId), mir::SymbId>,
}


pub struct MGen;

impl<'ast, 'hir, 'mir, 'a> MGen {

  pub fn low(src: &'hir hir::Krate, deps: &'hir hir::Deps, sin: &'ast StrInterner, layinfo: &'a LayoutInfo) -> (Option<mir::Krate>, Summary) {
    let mut cre = mir::Krate::new();
    let mut sum = Summary::new();
    let mut tin = TypeInterner::new(&mut cre, layinfo);
    let mut cmap = CacheMap {
      cache_type: FxHashMap::default(),
      cache_item: FxHashMap::default(),
      cache_vmt: FxHashMap::default(),
    };

    let root = src.root().unwrap();

    let _ = SymbLow::low(
      &mut Ctx{cre: &mut cre, tin: &mut tin, cmap: &mut cmap, src, deps, sin},
      root
    ).map_err(|msg| sum.add(msg));

    (Some(cre), sum)
  }

}

