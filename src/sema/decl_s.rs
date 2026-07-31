use crate::diagnostic::Message;
use super::Sema;
use crate::control::identy::IdentyId;
use crate::ast::DeclVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_decl(&mut self, id: IdentyId) -> Result<(), Message<'a>> {
    self.check_attributes(id)?;
    let decl = self.mol.get_decl(id);
    let vari = &decl.vari;

    match vari {
      DeclVari::Using(ty_id) => {
        let ty_id = *ty_id;
        self.check_type(ty_id)?;
      }
      
      DeclVari::Module(m) => {
        let decls = m.decls.clone();
        for did in decls {
          self.check_decl(did)?;
        }
      }
      
      DeclVari::Fun(f) => {
        let kind = f.kind;
        let blok = f.blok;
        self.check_type(kind)?;
        self.visitors.push(id);
        self.check_expr(blok)?;
        self.visitors.pop();
      }
      
      DeclVari::Var(v) => {
        let kind = v.kind;
        let init = v.init;
        self.check_type(kind)?;
        if let Some(i) = init {
          self.check_expr(i)?;
        }
      }
      
      DeclVari::Import(path, _) => {
        let path_strs: Vec<String> = path.iter().map(|(idx, _)| self.mol.nick_map[*idx as usize].clone()).collect();
        let pos = path[0].1;
        if let Some(res_id) = crate::sema::scopemng::ScopeManager::find(self.mol, self.mol.get_mod(), &path_strs) {
          if let DeclVari::Import(_, ref mut res) = self.mol.get_mut_decl(id).vari {
            *res = Some(res_id);
          }
        } else {
          return Err(Message::error(pos, format!("unknown import `{}`", path_strs.join("::")), vec![]));
        }
      }

      DeclVari::ImportWildcard(path, _) => {
        let path_strs: Vec<String> = path.iter().map(|(idx, _)| self.mol.nick_map[*idx as usize].clone()).collect();
        let pos = path[0].1;
        if let Some(res_id) = crate::sema::scopemng::ScopeManager::find(self.mol, self.mol.get_mod(), &path_strs) {
          if res_id.kind() != crate::control::IdentyKind::Decl {
             return Err(Message::error(pos, "wildcard import target must be a module".to_string(), vec![]));
          }
          if let DeclVari::ImportWildcard(_, ref mut res) = self.mol.get_mut_decl(id).vari {
            *res = Some(res_id);
          }
        } else {
          return Err(Message::error(pos, format!("unknown import `{}`", path_strs.join("::")), vec![]));
        }
      }
    }

    Ok(())
  }

}
