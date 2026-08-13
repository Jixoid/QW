use crate::control::{Module, identy::AstId};


pub struct ScopeManager;

impl ScopeManager {
  
  pub fn find_local(mol: &Module, visitors: &[AstId], target: &str) -> Option<AstId> {
    for &vid in visitors.iter().rev() {
      if vid.kind() == crate::control::IdentyKind::Expr {
        let expr = mol.get_expr(vid);
        if let crate::ast::ExprVari::Block(b) = &expr.vari {
          for &stmt_id in &b.ctn {
            let stmt = mol.get_stmt(stmt_id);
            if let crate::ast::StmtVari::Let(l) = &stmt.vari {
              if l.name.str() == target {
                return Some(stmt_id);
              }
            }
          }
        }
      }
    }
    None
  }

  
  pub fn find(mol: &Module, iid: AstId, name: &[String]) -> Option<AstId> {
    match iid.kind() {
      crate::control::IdentyKind::Decl => Self::find_in_decl(mol, iid, name),
      crate::control::IdentyKind::Type => Self::find_in_type(mol, iid, name),
      _ => todo!("expected decl or type"),
    }
  }


  pub fn find_in_type(mol: &Module, ty_id: AstId, name: &[String]) -> Option<AstId> {
    if name.is_empty() { return Some(ty_id); }
    
    if ty_id.module() != 0 {
      let dep = mol.get_dep(ty_id.module());
      let local_id = crate::control::identy::AstId::new(ty_id.kind(), 0, ty_id.index());
      if let Some(found_id) = Self::find_in_type(dep, local_id, name) {
        if found_id.module() == 0 {
          return Some(crate::control::identy::AstId::new(found_id.kind(), ty_id.module(), found_id.index()));
        } else {
          return Some(found_id);
        }
      }
      return None;
    }
    
    let ty = mol.get_type(ty_id);
    let target = &name[0];
    
    match &ty.vari {
      crate::ast::TypeVari::Enum(e) => {
        if e.vals.iter().any(|v| v.name.str() == target) {
          if name.len() == 1 {
            return Some(ty_id);
          }
        }
      }

      crate::ast::TypeVari::Flags(f) => {
        if f.vals.iter().any(|v| v.name.str() == target) {
          if name.len() == 1 {
            return Some(ty_id);
          }
        }
      }
      
      _ => println!("cannot finding in: {:?}", &ty)
    }
    
    None
  }

  pub fn find_in_decl(mol: &Module, decl_id: AstId, name: &[String]) -> Option<AstId> {
    if name.is_empty() { return None; }

    if decl_id.module() != 0 {
      let dep = mol.get_dep(decl_id.module());
      let local_id = crate::control::identy::AstId::new(decl_id.kind(), 0, decl_id.index());
      if let Some(found_id) = Self::find_in_decl(dep, local_id, name) {
        if found_id.module() == 0 {
          return Some(crate::control::identy::AstId::new(found_id.kind(), decl_id.module(), found_id.index()));
        } else {
          return Some(found_id);
        }
      }
      return None;
    }

    let decl = mol.get_decl(decl_id);
    let target = &name[0];

    match &decl.vari {
      crate::ast::DeclVari::Module(m) => {

        for &child_id in &m.decls {
          let mut local_child = child_id;
          if child_id.module() == 0 && decl_id.module() != 0 {
             local_child = crate::control::identy::AstId::new(child_id.kind(), decl_id.module(), child_id.index());
          }

          let child = mol.get_decl(local_child);
          
          let child_name = match &child.name {
            crate::ast::DeclName::Name(n) => n.as_str(),
            crate::ast::DeclName::Word(w) => w.str(),
          };

          if child_name == target {
            let resolved_id = match &child.vari {
              crate::ast::DeclVari::Using(uid) => {
                 let mut local_uid = *uid;
                 if uid.module() == 0 && decl_id.module() != 0 {
                    local_uid = crate::control::identy::AstId::new(uid.kind(), decl_id.module(), uid.index());
                 }
                 local_uid
              },
              crate::ast::DeclVari::Import(_, Some(uid)) => {
                 let mut local_uid = *uid;
                 if uid.module() == 0 && decl_id.module() != 0 {
                    local_uid = crate::control::identy::AstId::new(uid.kind(), decl_id.module(), uid.index());
                 }
                 local_uid
              },
              crate::ast::DeclVari::Import(_, None) => return None,
              _ => local_child,
            };

            if name.len() == 1 {
              return Some(resolved_id);
            } else {
              return Self::find(mol, resolved_id, &name[1..]);
            }
          }
        }

        // If not found directly, check wildcard imports
        for &child_id in &m.decls {
          let mut local_child = child_id;
          if child_id.module() == 0 && decl_id.module() != 0 {
             local_child = crate::control::identy::AstId::new(child_id.kind(), decl_id.module(), child_id.index());
          }

          let child = mol.get_decl(local_child);
          if let crate::ast::DeclVari::ImportWildcard(_, Some(uid)) = &child.vari {
            let mut local_uid = *uid;
            if uid.module() == 0 && decl_id.module() != 0 {
               local_uid = crate::control::identy::AstId::new(uid.kind(), decl_id.module(), uid.index());
            }

            if let Some(found_id) = Self::find(mol, local_uid, name) {
               return Some(found_id);
            }
          }
        }

        if decl_id == mol.get_mod() {
          for (i, dep) in mol.imod.iter().enumerate() {
            if dep.name == *target {
              if name.len() == 1 {
                return Some(crate::control::identy::AstId::new(crate::control::IdentyKind::Decl, (i as u32) + 1, 0));
              } else {
                let dep_id = crate::control::identy::AstId::new(crate::control::IdentyKind::Decl, (i as u32) + 1, 0);
                return Self::find(mol, dep_id, &name[1..]);
              }
            }
          }
        }
      }
    
      crate::ast::DeclVari::Using(u) => {
        return Self::find_in_type(mol, *u, name);
      }

      _ => {}
    }

    None
  }

}
