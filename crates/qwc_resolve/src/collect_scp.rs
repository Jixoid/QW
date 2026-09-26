use qwc_arena::Files;
use qwc_ast::{AnyId, Item, ItemId, ItemKind, Krate, Thing, ThingId, Type, TypeKind, Visitor};
use qwc_diagnostic::Summary;
use qwc_string_interner::StrInterner;

use crate::{Imod, Scope, ScopeKindAst, ScopeMap, import, scope::{ImportDef, ImportSegment}};


impl Visitor for ScopeMap {
  type Type = Self;

  fn visit(cre: &Krate, sin: &StrInterner, _: &Files) -> Result<Self::Type, Summary> {
    ScopeCollector::collect(cre, sin, &[])
  }
}


pub struct ScopeCollector<'a, 'imod> {
  cre: &'a Krate,
  sin: &'a StrInterner,
  imods: &'a [Imod<'imod>],
  scp: ScopeMap,
  sum: Summary,
}

impl<'a, 'imod> ScopeCollector<'a, 'imod> {

  pub fn collect(cre: &'a Krate, sin: &'a StrInterner, imods: &'a [Imod<'imod>]) -> Result<ScopeMap, Summary> {
    let mut collector = Self {
      cre,
      sin,
      imods,
      scp: ScopeMap::new(),
      sum: Summary::new(),
    };

    if let Some(root_id) = cre.root() {
      collector.process_container(root_id.to_any(), None);
    }

    if collector.sum.is_empty() {
      import::resolve_imports(&mut collector.scp, collector.cre, collector.sin, collector.imods, &mut collector.sum);
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
        ItemKind::Module(..) | ItemKind::ModuleFile(..) => ScopeKindAst::Module(id),
        
        ItemKind::Using(kind) | ItemKind::ItemTy(kind) => ScopeKindAst::Type(kind),
        
        ItemKind::Let{..} | ItemKind::Fun{..} => ScopeKindAst::Expr(id),

        // No Named / continue
        ItemKind::Import(rng) => {
          let mut segments = vec![];
          let mut glob = false;

          for id in self.cre.extra_get(rng) {
            let it: &Thing = self.cre.get(id);

            match it {
              Thing::Crate => segments.push(ImportSegment::Crate),
              Thing::Super => segments.push(ImportSegment::Super),
              Thing::Name(ident) => segments.push(ImportSegment::Name(*ident)),
              Thing::Wildcard => { glob = true; break }
              _ => todo!("{it:#?}")
            }
          }

          if segments.is_empty() && !glob { panic!("empty import path"); }

          current_scope.import.push(ImportDef::new(
            item.pos,
            item.vis,
            segments,
            glob,
          ));

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
            let it = ScopeKindAst::TypeParam(param_id);

            current_scope.insert(*name, it, &mut self.sum);
          }
          
          Thing::NamedType(name, kind) => {
            let ty_obj: &Type = self.cre.get(*kind);

            let it = match ty_obj.kind {
              TypeKind::Type() => ScopeKindAst::TypeParam(param_id),
              _ => ScopeKindAst::ExprParam(param_id),
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
        self.cre.extra_get(rng).collect()
      }
      
      ItemKind::Generic{ctn, ..} => {
        self.cre.extra_get(ctn).collect()
      }

      _ => vec![],
    }
  }

  fn get_generic_params(&self, id: AnyId) -> Option<Vec<ThingId>> {
    let item: &Item = self.cre.get(ItemId::from_any(id));
    if let ItemKind::Generic{params, ..} = item.kind {
      Some(self.cre.extra_get(params).collect())
    } else {
      None
    }
  }

}
