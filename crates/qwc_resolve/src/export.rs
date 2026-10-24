/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{AnyId, CID, ItemId, TypeId};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct ExportMap {
  pub(crate) cid: CID,
  pub(crate) root: Option<ItemId>,
  pub(crate) map: FxHashMap<AnyId, Export>,
  pub symbols: FxHashMap<ItemId, String>,
}

impl ExportMap {

  pub fn new(cid: CID, root: Option<ItemId>) -> Self {
    Self {
      cid,
      root,
      map: FxHashMap::default(),
      symbols: FxHashMap::default(),
    }
  }

  pub fn cid(&self) -> CID {
    self.cid
  }

  pub fn root(&self) -> Option<ItemId> {
    self.root
  }

  pub fn symbol(&self, id: &ItemId) -> Option<&str> {
    self.symbols.get(id).map(|s| s.as_str())
  }

  pub fn add_symbol(&mut self, id: ItemId, name: String) {
    self.symbols.insert(id, name);
  }

  pub fn get(&self, id: &AnyId) -> Option<&Export> {
    self.map.get(id)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&AnyId, &Export)> {
    self.map.iter()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  pub fn dump<'a>(&'a self, cre: &'a qwc_hir::Krate, sin: &'a qwc_string_interner::StrInterner, root: AnyId) -> crate::dump_exp::Dump<'a> {
    crate::dump_exp::Dump {
      exp: self,
      cre,
      deps: None,
      sin,
      root,
    }
  }

}


#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ExportKind {
  Type(TypeId),
  
  Expr(ItemId, TypeId),

  NameSpace(ItemId),
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Export {
  pub parent: Option<AnyId>,
  pub(crate) map: FxHashMap<String, ExportKind>,
}

impl Export {

  pub fn new(parent: Option<AnyId>) -> Self {
    Self {
      parent,
      map: FxHashMap::default(),
    }
  }

  pub fn insert(&mut self, name: String, kind: ExportKind) {
    self.map.insert(name, kind).inspect(|_| panic!());
  }

  pub fn get(&self, name: &str) -> Option<&ExportKind> {
    self.map.get(name)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&String, &ExportKind)> {
    self.map.iter()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

}
