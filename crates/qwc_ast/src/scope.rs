use std::num::NonZeroU32;

use rustc_hash::FxHashMap;

use crate::{AnyId, Item, ItemId, ItemKind, Krate, Thing, ThingId, Type, TypeKind, id::{AstId, NodeKind, SpecAny}};


pub enum ScopeKind {
  Expr(AnyId),
  Type(AnyId),
  Scope(AnyId),
}


pub struct Scope {
  map: FxHashMap<NonZeroU32, ScopeKind>,
}

impl Scope {

  pub fn new() -> Self {
    Self {
      map: FxHashMap::default(),
    }
  }

  pub fn new_with<R: ReadScope>(cre: &Krate, it: R) -> Option<Self> {
    R::read(cre, it)
  }


  pub fn insert(&mut self, id: NonZeroU32, kind: ScopeKind) {
    self.map.insert(id, kind);
  }

  pub fn get(&self, id: &NonZeroU32) -> Option<&ScopeKind> {
    self.map.get(id)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&NonZeroU32, &ScopeKind)> {
    self.map.iter()
  }

  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  pub fn len(&self) -> usize {
    self.map.len()
  }
}


pub trait ReadScope {
  fn read(cre: &Krate, it: Self) -> Option<Scope>;
}

impl ReadScope for Item {
  fn read(cre: &Krate, it: Self) -> Option<Scope> {
    match it.kind {
      ItemKind::Krate(rng) | ItemKind::Module(rng) => {
        let mut scp = Scope::new();

        for id in cre.extra_get(rng) {
          let it: &Item = cre.get(ItemId::new_from(id));

          collect_item(cre, &mut scp, id, it);
        }

        Some(scp)
      }
      
      ItemKind::Generic{ctn, params, ..} => {
        let mut scp = Scope::new();

        for id in cre.extra_get(params) {
          let (name, kind) = if let Thing::NamedType(name, kind) = cre.get(ThingId::new_from(id)) { (name, kind) } else { panic!() };

          if let TypeKind::Type() = (cre.get(*kind) as &Type).kind {
            scp.insert(name.sid(), ScopeKind::Type(AnyId::new_from(id)));
          } else {
            scp.insert(name.sid(), ScopeKind::Expr(AnyId::new_from(id)));
          }
        }

        for id in cre.extra_get(ctn) {
          let it: &Item = cre.get(ItemId::new_from(id));

          collect_item(cre, &mut scp, id, it);
        }

        Some(scp)
      }

      _ => todo!()
    }
  }
}


fn collect_item(cre: &Krate, scp: &mut Scope, id: (AstId<SpecAny>, NodeKind), it: &Item) {
  match it.kind {
    ItemKind::Using(..) | ItemKind::ItemTy(..) => {
      scp.insert(it.name.unwrap().sid(), ScopeKind::Type(AnyId::new_from(id)));
    }
    
    ItemKind::Fun{..} | ItemKind::Let {..} => {
      scp.insert(it.name.unwrap().sid(), ScopeKind::Expr(AnyId::new_from(id)));
    }
    
    ItemKind::Module(..) | ItemKind::ModuleFile(..) => {
      scp.insert(it.name.unwrap().sid(), ScopeKind::Scope(AnyId::new_from(id)));
    }

    ItemKind::Generic{ctn, ..} => {
      for id in cre.extra_get(ctn) {
        let it: &Item = cre.get(ItemId::new_from(id));

        collect_item(cre, scp, id, it);
      }
    }

    ItemKind::Import(..) => {},

    _ => todo!()
  }
}
