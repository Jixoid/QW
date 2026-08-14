use crate::diagnostic::Message;
use super::Sema;
use crate::control::identy::AstId;
use crate::ast::DeclVari;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_decl(&mut self, id: AstId) -> Result<(), Message> {
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
          let init_ty = self.check_expr(i)?;
          if kind.kind() != crate::control::IdentyKind::Null && !self.is_type_equal(kind, init_ty, None) {
            if let Some(pos) = self.mol.get_decl(id).name.pos().cloned() {
              return Err(Message::error(pos, "type mismatch in variable initialization".to_string(), vec![]));
            }
          }
        }
      }
      
      DeclVari::Extend(ext) => {
        let target = ext.target;
        let tr = ext.tr;
        let impls = ext.impls.clone();

        self.check_type(target)?;
        self.check_type(tr)?;

        for i in &impls {
          self.check_decl(*i)?;
        }

        // Resolve `tr` to its base Trait or Iface definition
        let mut actual_tr_id = tr;
        let mut trait_funs = None;

        for _ in 0..10 {
          let ty = self.mol.get_type(actual_tr_id);
          match &ty.vari {
            crate::ast::TypeVari::Path(p) => {
              if p.is_empty() { break; }
              if p[0].kind() == crate::control::IdentyKind::Type {
                actual_tr_id = p[0];
              } else {
                break;
              }
            }
            crate::ast::TypeVari::Trait(t) => {
              trait_funs = Some(t.funs.clone());
              break;
            }
            crate::ast::TypeVari::Iface(t) => {
              trait_funs = Some(t.funs.clone());
              break;
            }
            _ => break,
          }
        }

        if let Some(funs) = trait_funs {
           let tr_mod = actual_tr_id.module();

           for req_fun in funs {
              let mut found = false;
              let req_name = req_fun.name.str();

              let mut req_kind = req_fun.kind;
              if req_kind.module() == 0 && tr_mod != 0 {
                req_kind = crate::control::identy::AstId::new(req_kind.kind(), tr_mod, req_kind.index());
              }

              for &impl_id in &impls {
                let imp_decl = self.mol.get_decl(impl_id);
                if imp_decl.name.to_string() == req_name {
                   found = true;
                   if let DeclVari::Fun(f) = &imp_decl.vari {
                     if !self.is_type_equal(req_kind, f.kind, Some(target)) {
                       let imp_pos = imp_decl.name.pos().cloned().unwrap_or(req_fun.name);
                       return Err(Message::error(imp_pos, format!("signature mismatch for `{}`", req_name), vec![]));
                     }
                   }
                   break;
                }
             }
             if !found {
               let extend_kw = self.mol.get_decl(id).name.pos().cloned().unwrap_or(req_fun.name);
               return Err(Message::error(extend_kw, format!("missing implementation for `{}`", req_name), vec![]));
             }
          }
        } else {
           let extend_kw = self.mol.get_decl(id).name.pos().cloned().unwrap();
           return Err(Message::error(extend_kw, "extend requires a valid trait or iface".to_string(), vec![]));
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
      
      DeclVari::Generic(_g) => {
        // generic decl not checked yet
      }
    }

    Ok(())
  }

  
  pub fn resolve_type_id(&self, mut id: AstId) -> AstId {
    for _ in 0..10 {
      if id.kind() == crate::control::IdentyKind::Type {
        let ty = self.mol.get_type(id);
        match &ty.vari {
          crate::ast::TypeVari::Path(p) if !p.is_empty() => {
            if p[0].kind() == crate::control::IdentyKind::Type {
              id = p[0];
            } else {
              break;
            }
          }
          _ => break,
        }
      } else if id.kind() == crate::control::IdentyKind::Decl {
        let decl = self.mol.get_decl(id);
        match &decl.vari {
          crate::ast::DeclVari::Using(uid) => {
            id = *uid;
          }
          _ => break,
        }
      } else {
        break;
      }
    }
    id
  }

  pub fn current_fn_ret_type(&self) -> Option<AstId> {
    for &id in self.visitors.iter().rev() {
      if id.kind() == crate::control::IdentyKind::Decl {
        let decl = self.mol.get_decl(id);
        if let crate::ast::DeclVari::Fun(f) = &decl.vari {
          let ty = self.mol.get_type(f.kind);
          if let crate::ast::TypeVari::Function(ft) = &ty.vari {
            let mut ret_ty = ft.ret;
            if ret_ty.module() == 0 && id.module() != 0 {
              ret_ty = crate::control::identy::AstId::new(ret_ty.kind(), id.module(), ret_ty.index());
            }
            return Some(ret_ty);
          }
        }
      }
    }
    None
  }

  pub fn is_type_equal(&self, a: AstId, b: AstId, self_ty: Option<AstId>) -> bool {
    if a == b { return true; }

    let a = self.resolve_type_id(a);
    let b = self.resolve_type_id(b);
    if a == b { return true; }
    
    let ta = self.mol.get_type(a);
    let tb = self.mol.get_type(b);
    
    let loc = |id: crate::control::identy::AstId, p_mod: u32| -> crate::control::identy::AstId {
      if id.module() == 0 && p_mod != 0 {
        crate::control::identy::AstId::new(id.kind(), p_mod, id.index())
      } else { id }
    };
    
    match (&ta.vari, &tb.vari) {
      (crate::ast::TypeVari::SelfType, _) if self_ty.is_some() => self.is_type_equal(self_ty.unwrap(), b, self_ty),
      (_, crate::ast::TypeVari::SelfType) if self_ty.is_some() => self.is_type_equal(a, self_ty.unwrap(), self_ty),
      
      (crate::ast::TypeVari::Int{bit: b1, sig: s1}, crate::ast::TypeVari::Int{bit: b2, sig: s2}) => b1 == b2 && s1 == s2,
      (crate::ast::TypeVari::Float{bit: b1}, crate::ast::TypeVari::Float{bit: b2}) => b1 == b2,
      (crate::ast::TypeVari::ArchSize{sig: s1}, crate::ast::TypeVari::ArchSize{sig: s2}) => s1 == s2,
      (crate::ast::TypeVari::Bool, crate::ast::TypeVari::Bool) => true,
      (crate::ast::TypeVari::Char, crate::ast::TypeVari::Char) => true,
      (crate::ast::TypeVari::Ptr, crate::ast::TypeVari::Ptr) => true,
      (crate::ast::TypeVari::Void, crate::ast::TypeVari::Void) => true,
      (crate::ast::TypeVari::Null, crate::ast::TypeVari::Null) => true,
      (crate::ast::TypeVari::SelfType, crate::ast::TypeVari::SelfType) => true,
      (crate::ast::TypeVari::Path(p1), crate::ast::TypeVari::Path(p2)) => {
        p1 == p2
      }
      (crate::ast::TypeVari::PointerOf{sub: s1, ..}, crate::ast::TypeVari::PointerOf{sub: s2, ..}) => self.is_type_equal(loc(*s1, a.module()), loc(*s2, b.module()), self_ty),
      (crate::ast::TypeVari::ArrayOf{sub: s1, ..}, crate::ast::TypeVari::ArrayOf{sub: s2, ..}) => self.is_type_equal(loc(*s1, a.module()), loc(*s2, b.module()), self_ty) && s1 == s2,
      (crate::ast::TypeVari::Function(f1), crate::ast::TypeVari::Function(f2)) => {
        if f1.args.len() != f2.args.len() { return false; }
        for (i, a1) in f1.args.iter().enumerate() {
          if !self.is_type_equal(loc(a1.kind, a.module()), loc(f2.args[i].kind, b.module()), self_ty) { return false; }
        }
        self.is_type_equal(loc(f1.ret, a.module()), loc(f2.ret, b.module()), self_ty)
      }
      _ => false
    }
  }

}
