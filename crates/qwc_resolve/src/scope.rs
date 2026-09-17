use qwc_ast::{AnyId, Ident, ItemId, ThingId, TypeId};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;
use qwc_diagnostic::{Label, Message, Span, Summary, msg::*};


pub struct ScopeMap {
  pub(crate) map: FxHashMap<AnyId, Scope>,
}

impl ScopeMap {

  pub fn new() -> Self {
    Self {
      map: FxHashMap::default(),
    }
  }

  pub fn get(&self, id: &AnyId) -> Option<&Scope> {
    self.map.get(id)
  }

}



#[derive(Clone, Copy, Debug)]
pub enum ScopeKind {
  Type(TypeId),
  TypeParam(ThingId),
  
  Expr(ItemId),
  ExprParam(ThingId),

  Module(ItemId),
}

pub enum ImportDef {
  Unsolved{
    path: Vec<Ident>,
    glob: bool,
  },
}



pub struct Scope {
  pub parent: Option<AnyId>,
  pub import: Vec<ImportDef>,
  pub(crate) map: FxHashMap<Sid, (ScopeKind, Span)>,
}

impl Scope {

  pub fn new(parent: Option<AnyId>) -> Self {
    Self {
      parent,
      import: vec![],
      map: FxHashMap::default(),
    }
  }

  pub fn insert(&mut self, ident: Ident, kind: ScopeKind, sum: &mut Summary) {
    use std::collections::hash_map::Entry;

    match self.map.entry(ident.sid()) {
      Entry::Occupied(entry) => {
        let (_, _) = entry.get();
        sum.add(Message::error(DUPLICATE_IDENTIFIER, Label::new_pos(ident)));
      }
      Entry::Vacant(entry) => {
        entry.insert((kind, ident.into()));
      }
    }
  }

  pub fn get(&self, sid: &Sid) -> Option<&(ScopeKind, Span)> {
    self.map.get(sid)
  }

}
