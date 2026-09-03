use std::num::NonZeroU32;

use crate::{ast, hgen::GenContext, lexer::SrcLoc};


pub struct Global {
  scopes: Vec<ast::AnyId>,
}


impl Global {

  pub fn new() -> Self {
    Self{
      scopes: vec![],
    }
  }


  pub fn enter<T>(&mut self, id: ast::AstId<T>) {
    self.scopes.push(id.to_any());
  }

  pub fn leave(&mut self) {
    self.scopes.pop();
  }


  pub fn lookup(&self, ctx: &GenContext, sid: NonZeroU32) -> Option<ast::AnyId> {
    let name = ctx.ast.get_sid(sid);
    let mut visited = std::collections::HashSet::new();

    for &id in self.scopes.iter().rev() {
      if let Some(found) = self.lookup_in_scope_visited(ctx, id, name, &mut visited) {
        return Some(found);
      }
    }
    
    // Fallback: search in crate root
    let root_any = ctx.ast.root.to_any();
    if !self.scopes.contains(&root_any) {
      if let Some(found) = self.lookup_in_scope_visited(ctx, root_any, name, &mut visited) {
        return Some(found);
      }
    }

    None
  }

  pub fn lookup_name(&self, ctx: &GenContext, name: &str) -> Option<ast::AnyId> {
    let mut visited = std::collections::HashSet::new();

    for &id in self.scopes.iter().rev() {
      if let Some(found) = self.lookup_in_scope_visited(ctx, id, name, &mut visited) {
        return Some(found);
      }
    }
    
    let root_any = ctx.ast.root.to_any();
    if !self.scopes.contains(&root_any) {
      if let Some(found) = self.lookup_in_scope_visited(ctx, root_any, name, &mut visited) {
        return Some(found);
      }
    }

    None
  }

  pub fn lookup_in_scope(&self, ctx: &GenContext, scope_id: ast::AnyId, sid: NonZeroU32) -> Option<ast::AnyId> {
    let name = ctx.ast.get_sid(sid);
    let mut visited = std::collections::HashSet::new();
    
    self.lookup_in_scope_visited(ctx, scope_id, name, &mut visited)
  }


  fn lookup_in_scope_visited(&self, ctx: &GenContext, scope_id: ast::AnyId, name: &str, visited: &mut std::collections::HashSet<ast::AnyId>) -> Option<ast::AnyId> {
    if !visited.insert(scope_id) { return None; }

    let scp = ctx.ast.get_scope(scope_id)?;

    // Direct map lookup (by string equality)
    for (k, v) in &scp.map {
      if ctx.ast.get_sid(*k) == name {
        return Some(*v);
      }
    }

    // Direct search in item contents
    if scope_id.kind == ast::AstKind::Item {
      let item = ctx.ast.get_item(ast::ItemId::new_from(scope_id));
      
      let ctn = match &item.vari {
        ast::ItemVari::Generic { ctn, .. } | ast::ItemVari::Impl { ctn, .. } | ast::ItemVari::Module { ctn, .. } => ctn,
        _ => &(0..0),
      };

      for &child_id in ctx.ast.get_extra(ctn) {
        if child_id.kind == ast::AstKind::Decl {
          let decl = ctx.ast.get_decl(ast::DeclId::new_from(child_id));
          
          if decl.name.str(ctx.far) == name { return Some(child_id); }
        }
      }
    }

    // Search
    if scope_id.kind == ast::AstKind::Item {
      let item = ctx.ast.get_item(ast::ItemId::new_from(scope_id));
      
      if let ast::ItemVari::Module { ctn, .. } = &item.vari {
        for &child_id in ctx.ast.get_extra(ctn) {
          if child_id.kind == ast::AstKind::Item {
            let child_item = ctx.ast.get_item(ast::ItemId::new_from(child_id));
            if let ast::ItemVari::Import(import_rng) = &child_item.vari {
              let segs = ctx.ast.get_extra(import_rng);
              if segs.is_empty() { continue; }

              let last_seg = ctx.ast.get_thing(ast::ThingId::new_from(segs[segs.len() - 1]));
              match last_seg {
                ast::Thing::Wildcard => {
                  let mut target = None;

                  for (i, &seg_id) in segs[..segs.len() - 1].iter().enumerate() {
                    let seg_thing = ctx.ast.get_thing(ast::ThingId::new_from(seg_id));
                    if let ast::Thing::Name(span) = seg_thing {
                      let seg_name = span.str(ctx.far);
                      if i == 0 {
                        for (k, v) in &scp.map {
                          if ctx.ast.get_sid(*k) == seg_name {
                            target = Some(*v);
                            break;
                          }
                        }
                        if target.is_none() {
                          if let Some(root_scp) = ctx.ast.get_scope(ctx.ast.root.to_any()) {
                            for (k, v) in &root_scp.map {
                              if ctx.ast.get_sid(*k) == seg_name {
                                target = Some(*v);
                                break;
                              }
                            }
                          }
                        }
                      } else if let Some(t) = target {
                        if let Some(t_scp) = ctx.ast.get_scope(t) {
                          target = None;
                          for (k, v) in &t_scp.map {
                            if ctx.ast.get_sid(*k) == seg_name {
                              target = Some(*v);
                              break;
                            }
                          }
                        } else {
                          target = None;
                        }
                      }
                    }
                  }

                  if let Some(target_mod) = target {
                    if let Some(found) = self.lookup_in_scope_visited(ctx, target_mod, name, visited) {
                      return Some(found);
                    }
                  }
                }
                
                ast::Thing::Name(span) => {
                  let import_alias = span.str(ctx.far);
                  if import_alias == name {
                    let mut target = None;
                    for (i, &seg_id) in segs.iter().enumerate() {
                      let seg_thing = ctx.ast.get_thing(ast::ThingId::new_from(seg_id));
                      if let ast::Thing::Name(s) = seg_thing {
                        let s_name = s.str(ctx.far);
                        if i == 0 {
                          for (k, v) in &scp.map {
                            if ctx.ast.get_sid(*k) == s_name {
                              target = Some(*v);
                              break;
                            }
                          }
                          if target.is_none() {
                            if let Some(root_scp) = ctx.ast.get_scope(ctx.ast.root.to_any()) {
                              for (k, v) in &root_scp.map {
                                if ctx.ast.get_sid(*k) == s_name {
                                  target = Some(*v);
                                  break;
                                }
                              }
                            }
                          }
                        } else if let Some(t) = target {
                          if let Some(t_scp) = ctx.ast.get_scope(t) {
                            target = None;
                            for (k, v) in &t_scp.map {
                              if ctx.ast.get_sid(*k) == s_name {
                                target = Some(*v);
                                break;
                              }
                            }
                          } else {
                            target = None;
                          }
                        }
                      }
                    }

                    if let Some(found) = target {
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
