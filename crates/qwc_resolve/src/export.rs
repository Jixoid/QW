use qwc_hir::{AnyId, CID, ItemId, TypeId};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;
use serde::Serialize;


#[derive(Serialize)]
pub struct ExportMap {
  pub(crate) cid: CID,
  pub(crate) root: Option<ItemId>,
  pub(crate) map: FxHashMap<AnyId, Export>,
}

impl ExportMap {

  pub fn new(cid: CID, root: Option<ItemId>) -> Self {
    Self {
      cid,
      root,
      map: FxHashMap::default(),
    }
  }

  pub fn cid(&self) -> CID {
    self.cid
  }

  pub fn root(&self) -> Option<ItemId> {
    self.root
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
      sin,
      root,
    }
  }

}


#[derive(Serialize, Clone, Copy, Debug)]
pub enum ExportKind {
  Type(TypeId),
  //TypeParam(ThingId),
  
  Expr(ItemId, TypeId),
  //ExprParam(ThingId),

  NameSpace(ItemId),
}


#[derive(Serialize)]
pub struct Export {
  pub parent: Option<AnyId>,
  pub(crate) map: FxHashMap<Sid, ExportKind>,
}

impl Export {

  pub fn new(parent: Option<AnyId>) -> Self {
    Self {
      parent,
      map: FxHashMap::default(),
    }
  }

  pub fn insert(&mut self, sid: Sid, kind: ExportKind) {
    self.map.insert(sid, kind).inspect(|_| panic!());
  }

  pub fn get(&self, sid: &Sid) -> Option<&ExportKind> {
    self.map.get(sid)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&Sid, &ExportKind)> {
    self.map.iter()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

}
