use qwc_arena::Files;
use qwc_ast::{AnyId, Item, ItemId, ItemKind, Krate, Thing, ThingId, Type, TypeKind, Visitor};
use qwc_diagnostic::Summary;
use qwc_string_interner::StrInterner;

use crate::{Scope, ScopeKind, ScopeMap, scope::ImportDef};


impl Visitor for ScopeMap {
  type Type = Self;

  fn visit(cre: &Krate, _: &StrInterner, _: &Files) -> Result<Self::Type, Summary> {
    ScopeCollector::collect(cre)
  }
}


pub struct ScopeCollector<'a> {
  cre: &'a Krate,
  scp: ScopeMap,
  sum: Summary,
}

impl<'a> ScopeCollector<'a> {

  pub fn collect(cre: &'a Krate) -> Result<ScopeMap, Summary> {
    let mut collector = Self {
      cre,
      scp: ScopeMap::new(),
      sum: Summary::new(),
    };

    if let Some(root_id) = cre.root() {
      collector.process_container(root_id.to_any(), None);
    }

    if collector.sum.is_empty() {
      Ok(collector.scp)
    } else {
      Err(collector.sum)
    }
  }

  fn process_container(&mut self, container_id: AnyId, parent_scope: Option<AnyId>) {
    let mut current_scope = Scope::new(parent_scope);

    // Kapsayıcının içindeki item ID listesini al
    let item_ids = self.get_container_items(container_id);

    // Önce tüm yerel elemanları bu kapsama kaydet (İleriye dönük referanslar için)
    for &id in &item_ids {
      let item: &Item = self.cre.get(id);
      
      let kind = match item.kind {
        ItemKind::Module(..) | ItemKind::ModuleFile(..) => ScopeKind::Module(id),
        
        ItemKind::Using(kind ) | ItemKind::ItemTy(kind) => ScopeKind::Type(kind),
        
        ItemKind::Let{..} | ItemKind::Fun{..} => ScopeKind::Expr(id),

        // No Named / continue
        ItemKind::Import(rng) => {
          let mut path = vec![];
          let mut glob = false;

          for id in self.cre.extra_get(rng) {
            let it: &Thing = self.cre.get(ThingId::new_from(id));

            match it {
              Thing::Name(ident) => path.push(*ident),
              Thing::Wildcard => { glob = true; break }
              _ => todo!()
            }
          }

          if path.is_empty() { panic!() }

          current_scope.import.push(ImportDef::Unsolved{
            path,
            glob
          });

          continue
        }
        
        // Ignore
        ItemKind::Generic{..} => continue,

        _ => todo!("{item:#?}"),
      };
      current_scope.insert(item.name.unwrap(), kind, &mut self.sum);
    }

    // Generic parametreleri varsa onları da mevcut kapsama ekle
    if let Some(params) = self.get_generic_params(container_id) {
      for param_id in params {

        match self.cre.get(param_id) as &Thing {
          Thing::Name(name) => {
            let it = ScopeKind::TypeParam(param_id);

            current_scope.insert(*name, it, &mut self.sum);
          }
          
          Thing::NamedType(name, kind) => {
            let ty_obj: &Type = self.cre.get(*kind);

            let it = match ty_obj.kind {
              TypeKind::Type() => ScopeKind::TypeParam(param_id),
              _ => ScopeKind::ExprParam(param_id),
            };

            current_scope.insert(*name, it, &mut self.sum);
          }

          _ => panic!()
        }
      }
    }

    // Kapsamı kaydet
    self.scp.map.insert(container_id, current_scope);

    // Şimdi alt kapsayıcıların (iç modüller, generic'ler) içine gir (Özyineleme)
    for &id in &item_ids {
      let item: &Item = self.cre.get(id);
      match item.kind {
        ItemKind::Module(..) | ItemKind::ModuleFile(..) | ItemKind::Generic { .. } => {
          // Bir alt kapsayıcıya geçerken ebeveyn olarak mevcut container_id'yi ver
          self.process_container(id.to_any(), Some(container_id));
        }
        
        _ => {}
      }
    }

  }
  

  fn get_container_items(&self, id: AnyId) -> Vec<ItemId> {
    let item: &Item = self.cre.get(ItemId::from_any(id));
    match item.kind {
      ItemKind::Krate(rng) | ItemKind::Module(rng) | ItemKind::ModuleFile(rng, ..) => {
        self.cre.extra_get(rng).map(ItemId::new_from).collect()
      }
      
      ItemKind::Generic{ctn, ..} => {
        self.cre.extra_get(ctn).map(ItemId::new_from).collect()
      }

      _ => vec![],
    }
  }

  fn get_generic_params(&self, id: AnyId) -> Option<Vec<ThingId>> {
    let item: &Item = self.cre.get(ItemId::from_any(id));
    if let ItemKind::Generic { params, .. } = item.kind {
      Some(self.cre.extra_get(params).map(ThingId::new_from).collect())
    } else {
      None
    }
  }

}
