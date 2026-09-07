use qwc_hir::{AnyId, ItemId, TypeId};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;


pub struct ExportMap {
  pub(crate) map: FxHashMap<AnyId, Export>,
}

impl ExportMap {

  pub fn new() -> Self {
    Self {
      map: FxHashMap::default(),
    }
  }

  pub fn get(&self, id: &AnyId) -> Option<&Export> {
    self.map.get(id)
  }

}



#[derive(Clone, Copy, Debug)]
pub enum ExportKind {
  Type(TypeId),
  //TypeParam(ThingId),
  
  Expr(ItemId),
  //ExprParam(ThingId),

  NameSpace(ItemId),
}


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

}
