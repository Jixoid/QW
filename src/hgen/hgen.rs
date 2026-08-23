use crate::{ast, diagnostic::Message, hgen::ItemGen, hir, route::build::FileArena};


pub struct GenContext<'a,'h,'d> {
  pub ast: &'a ast::Crate,
  pub hir: &'h mut hir::Crate<'d>,
  pub far: &'a FileArena,
  pub scopes: Vec<ast::AnyId>,
  pub self_type: Option<ast::TypeId>,
  pub type_cache: std::collections::HashMap<ast::TypeId, hir::TypeId>,
  pub decl_cache: std::collections::HashMap<ast::DeclId, hir::TypeId>,
}

impl<'a,'h,'d> GenContext<'a,'h,'d> {

  pub fn enter_scope(&mut self, id: ast::AnyId) {
    self.scopes.push(id);
  }

  pub fn leave_scope(&mut self) {
    self.scopes.pop();
  }

  pub fn lookup(&self, sid: u32) -> Option<ast::AnyId> {
    let mut visited = std::collections::HashSet::new();

    for &id in self.scopes.iter().rev() {
      if let Some(found) = self.lookup_in_scope_visited(id, sid, &mut visited) {
        return Some(found);
      }
    }
    
    // Fallback: search in crate root
    let root_any = self.ast.root.to_any();
    if !self.scopes.contains(&root_any) {
      if let Some(found) = self.lookup_in_scope_visited(root_any, sid, &mut visited) {
        return Some(found);
      }
    }

    None
  }

  pub fn lookup_in_scope(&self, scope_id: ast::AnyId, sid: u32) -> Option<ast::AnyId> {
    let mut visited = std::collections::HashSet::new();
    self.lookup_in_scope_visited(scope_id, sid, &mut visited)
  }

  fn lookup_in_scope_visited(&self, scope_id: ast::AnyId, sid: u32, visited: &mut std::collections::HashSet<ast::AnyId>) -> Option<ast::AnyId> {
    if !visited.insert(scope_id) {
      return None;
    }

    let scp = self.ast.get_scope(scope_id)?;

    // Direct map lookup
    if let Some(&found) = scp.map.get(&sid) {
      return Some(found);
    }

    // Search
    if scope_id.kind == ast::AstKind::Item {
      let item = self.ast.get_item(ast::ItemId::new_from(scope_id));
      if let ast::ItemVari::Module { ctn, .. } = &item.vari {
        for &child_id in self.ast.get_extra(ctn) {
          if child_id.kind == ast::AstKind::Item {
            let child_item = self.ast.get_item(ast::ItemId::new_from(child_id));
            if let ast::ItemVari::Import(import_rng) = &child_item.vari {
              let segs = self.ast.get_extra(import_rng);
              if segs.is_empty() { continue; }

              let last_seg = self.ast.get_thing(ast::ThingId::new_from(segs[segs.len() - 1]));
              match last_seg {
                ast::Thing::Wildcard => {
                  let mut target = None;

                  for (i, &seg_id) in segs[..segs.len() - 1].iter().enumerate() {
                    let seg_thing = self.ast.get_thing(ast::ThingId::new_from(seg_id));
                    if let ast::Thing::Name(span) = seg_thing {
                      if i == 0 {
                        target = scp.map.get(&span.sid).copied();
                        if target.is_none() {
                          if let Some(root_scp) = self.ast.get_scope(self.ast.root.to_any()) {
                            target = root_scp.map.get(&span.sid).copied();
                          }
                        }
                      } else if let Some(t) = target {
                        if let Some(t_scp) = self.ast.get_scope(t) {
                          target = t_scp.map.get(&span.sid).copied();
                        } else {
                          target = None;
                        }
                      }
                    }
                  }

                  if let Some(target_mod) = target {
                    if let Some(found) = self.lookup_in_scope_visited(target_mod, sid, visited) {
                      return Some(found);
                    }
                  }
                }
                
                ast::Thing::Name(span) if span.sid == sid => {
                  let mut target = None;

                  for (i, &seg_id) in segs[..segs.len() - 1].iter().enumerate() {
                    let seg_thing = self.ast.get_thing(ast::ThingId::new_from(seg_id));
                    if let ast::Thing::Name(s) = seg_thing {
                      if i == 0 {
                        target = scp.map.get(&s.sid).copied();
                        if target.is_none() {
                          if let Some(root_scp) = self.ast.get_scope(self.ast.root.to_any()) {
                            target = root_scp.map.get(&s.sid).copied();
                          }
                        }
                      } else if let Some(t) = target {
                        if let Some(t_scp) = self.ast.get_scope(t) {
                          target = t_scp.map.get(&s.sid).copied();
                        } else {
                          target = None;
                        }
                      }
                    }
                  }

                  if let Some(target_mod) = target {
                    if let Some(found) = self.lookup_in_scope_visited(target_mod, sid, visited) {
                      return Some(found);
                    }
                  }
                }
                
                _ => {}
              }
            }
          }
        }
      }
    }

    None
  }

}


pub struct HGen;

impl HGen {

  pub fn lower<'a,'d>(ast: &'a ast::Crate, far: &'a FileArena) -> Result<hir::Crate<'d>, Message> {
    let mut ret = hir::Crate::new();
    
    let mut gctx = GenContext {
      ast,
      hir: &mut ret,
      far,
      scopes: vec![],
      self_type: None,
      type_cache: std::collections::HashMap::new(),
      decl_cache: std::collections::HashMap::new(),
    };

    ret.root = ItemGen::low_item(&mut gctx, ast.root)?;

    Ok(ret)
  }

}
