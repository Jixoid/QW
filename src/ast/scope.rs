use std::collections::HashMap;

use crate::ast::AnyId;


pub struct Scope {
  pub map: HashMap<u32, AnyId>,
}

impl Scope {
  
  pub fn new() -> Self {
    Self { map: HashMap::new() }
  }

  pub fn insert(&mut self, sid: u32, id: AnyId) {
    self.map.insert(sid, id);
  }

  pub fn from_anys(cre: &crate::ast::Crate, anys: &[AnyId]) -> Self {
    let mut scope = Self::new();
    for &id in anys {
      match id.kind {
        crate::ast::AstKind::Decl => {
          let decl = cre.get_decl(crate::ast::DeclId::new_from(id));
          scope.insert(decl.name.sid, id);
        }
        crate::ast::AstKind::Item => {
          let item = cre.get_item(crate::ast::ItemId::new_from(id));
          match &item.vari {
            crate::ast::ItemVari::Module { name, .. } => {
              if let Some(pos) = cre.str_pool.iter().position(|s| s == name) {
                scope.insert(pos as u32, id);
              }
            }
            crate::ast::ItemVari::Generic { ctn, .. } => {
              for &child in cre.get_extra(ctn) {
                if child.kind == crate::ast::AstKind::Decl {
                  let decl = cre.get_decl(crate::ast::DeclId::new_from(child));
                  scope.insert(decl.name.sid, child);
                }
              }
            }
            _ => {}
          }
        }
        crate::ast::AstKind::Thing => {
          let thing = cre.get_thing(crate::ast::ThingId::new_from(id));
          match thing {
            crate::ast::Thing::NamedType(span, ..) => {
              scope.insert(span.sid, id);
            }
            crate::ast::Thing::NamedTypeVis(span, ..) => {
              scope.insert(span.sid, id);
            }
            crate::ast::Thing::NamedExpr(span, ..) => {
              scope.insert(span.sid, id);
            }
            crate::ast::Thing::NamedTypeList(span, ..) => {
              scope.insert(span.sid, id);
            }
            crate::ast::Thing::Name(span) => {
              scope.insert(span.sid, id);
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
    scope
  }

  pub fn from_rng(cre: &crate::ast::Crate, rng: &crate::ast::Rng) -> Self {
    let anys = cre.get_extra(rng);
    Self::from_anys(cre, anys)
  }

}

