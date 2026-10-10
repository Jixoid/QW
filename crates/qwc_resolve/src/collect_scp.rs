/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_arena::Files;
use qwc_ast::{AnyId, Item, ItemId, ItemKind, Krate, ThingId, ThingKind, TypeKind, Visitor};
use qwc_diagnostic::Summary;
use qwc_string_interner::StrInterner;

use crate::{Imod, Scope, ScopeKindAst, ScopeMap, import, scope::{ImportDef, ImportSegment}};


impl Visitor for ScopeMap {
  type Type = (Self, Vec<ImplFor>);

  fn visit(cre: &Krate, sin: &StrInterner, _: &Files) -> Result<Self::Type, Summary> {
    ScopeCollector::collect(cre, sin, &[])
  }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImplFor {pub container: AnyId, pub item: ItemId}

pub struct ScopeCollector<'a, 'imod> {
  cre: &'a Krate,
  sin: &'a StrInterner,
  imods: &'a [Imod<'imod>],
  scp: ScopeMap,
  sum: Summary,
  impl_for: Vec<ImplFor>,
}

impl<'a, 'imod> ScopeCollector<'a, 'imod> {

  pub fn collect(cre: &'a Krate, sin: &'a StrInterner, imods: &'a [Imod<'imod>]) -> Result<(ScopeMap, Vec<ImplFor>), Summary> {
    let mut collector = Self { cre, sin, imods, scp: ScopeMap::new(), sum: Summary::new(), impl_for: vec![] };

    let root = cre.root().unwrap();

    collector.process_container(root.to_any(), None);
    

    if collector.sum.is_empty() {
      import::resolve_imports(&mut collector.scp, collector.cre, collector.sin, collector.imods, &mut collector.sum);
    }

    if collector.sum.is_empty() {
      Ok((collector.scp, collector.impl_for))
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
      let it = self.cre.get(id);
      
      let kind = match it.kind {
        ItemKind::Module(..) | ItemKind::ModuleFile(..) => ScopeKindAst::Module(id),
        
        ItemKind::Let{..} | ItemKind::Fun{..} | ItemKind::Task{..} => ScopeKindAst::Expr(id),
        
        ItemKind::Using(kind) | ItemKind::ItemTy(kind) => ScopeKindAst::Type(kind),

        // Record Impl
        ItemKind::Impl{..} => {
          self.impl_for.push(ImplFor{ container: container_id, item: id });
          continue
        }

        // No Named / continue
        ItemKind::Import(rng) => {
          let mut segments = vec![];
          let mut glob = false;
          let mut alias = None;

          for id in self.cre.extra_get(rng) {
            match self.cre.get(id).kind {
              ThingKind::Crate => segments.push(ImportSegment::Crate),
              ThingKind::Super => segments.push(ImportSegment::Super),
              ThingKind::Name(ident) => segments.push(ImportSegment::Name(ident)),
              ThingKind::Wildcard => { glob = true; break }
              ThingKind::Alias(inner_id, alias_ident) => {
                alias = Some(alias_ident);
                match self.cre.get(inner_id).kind {
                  ThingKind::Crate => segments.push(ImportSegment::Crate),
                  ThingKind::Super => segments.push(ImportSegment::Super),
                  ThingKind::Name(ident) => segments.push(ImportSegment::Name(ident)),
                  _ => panic!("invalid aliased thing")
                }
              }
              it @_ => todo!("{it:#?}")
            }
          }

          if segments.is_empty() && !glob { panic!("empty import path"); }

          current_scope.import.push(ImportDef::new(it.pos, it.vis, segments, glob, alias));
          continue
        }
        
        // Generic
        ItemKind::Generic{ctn, ..} => {
          for child_id in self.cre.extra_get(ctn) {
            let child = self.cre.get(child_id);
            let child_kind = match child.kind {
              ItemKind::Using(kind) | ItemKind::ItemTy(kind) => ScopeKindAst::GenericType(id, kind),
              ItemKind::Fun{..} | ItemKind::Task{..} | ItemKind::Let{..} => ScopeKindAst::GenericExpr(id, child_id),
              _ => continue,
            };
            if let Some(child_name) = child.name {
              current_scope.insert(child_name, child_kind, &mut self.sum);
            }
          }
          continue;
        }

        _ => todo!("{it:#?}"),
      };
      current_scope.insert(it.name.unwrap(), kind, &mut self.sum);
    }

    // Generic parametreleri varsa onları da mevcut kapsama ekle
    if let Some(params) = self.get_generic_params(container_id) {
      for (idx, &param_id) in params.iter().enumerate() {

        match self.cre.get(param_id).kind {
          ThingKind::Name(name) => {
            let it = ScopeKindAst::TypeParam(idx, param_id);

            current_scope.insert(name, it, &mut self.sum);
          }
          
          ThingKind::NamedType(name, kind) => {
            let ty_obj = self.cre.get(kind);

            let it = match ty_obj.kind {
              TypeKind::Type => ScopeKindAst::TypeParam(idx, param_id),
              _ => ScopeKindAst::ExprParam(idx, param_id),
            };

            current_scope.insert(name, it, &mut self.sum);
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
    let item = self.cre.get(ItemId::from_any(id));
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
    let item = self.cre.get(ItemId::from_any(id));
    if let ItemKind::Generic{params, ..} = item.kind {
      Some(self.cre.extra_get(params).collect())
    } else {
      None
    }
  }

}
