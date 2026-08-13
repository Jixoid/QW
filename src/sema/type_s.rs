use crate::diagnostic::{Message, MsgKind};
use crate::sema::scopemng::ScopeManager;
use super::Sema;
use crate::control::identy::AstId;
use crate::ast::{TypeVari, TypeState};
use std::mem;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_type(&mut self, id: AstId) -> Result<(), Message<'a>> {
    self.check_attributes(id)?;
    let state = self.mol.get_type(id).state;
    
    if state == TypeState::Resolved { return Ok(()); }
    if state == TypeState::Resolving {
      let pos = get_word_from_type(self.mol, id).or_else(|| {
        self.mol.list_type.iter().find_map(|t| {
          if let TypeVari::Nick(n) = &t.vari { Some(n.pos) } else { None }
        })
      }).expect("Cannot find any Word in module for error reporting");
      
      return Err(Message::new(MsgKind::Fatal, pos, format!("cyclic type dependency detected on type id {}", id), vec![]));
    }

    self.mol.get_mut_type(id).state = TypeState::Resolving;

    let mut vari = mem::replace(&mut self.mol.get_mut_type(id).vari, TypeVari::Null);

    match &mut vari {
      TypeVari::Nick(s) => {
        let name_str = self.mol.nick_map[s.idx as usize].clone();
        
        if let Some(res_id) = ScopeManager::find(self.mol, self.mol.get_mod(), &vec![name_str.clone()]) {
          vari = TypeVari::Path(vec![res_id]);
        } else {
          return Err(Message::new(MsgKind::Error, s.pos, "unknown type".to_string(), vec![name_str]));
        }
      }
      
      TypeVari::UnresolvedPath(p) => {
        let mut path_strs = Vec::new();
        for n in p.iter() {
          path_strs.push(self.mol.nick_map[n.idx as usize].clone());
        }
        
        if let Some(res_id) = ScopeManager::find(self.mol, self.mol.get_mod(), &path_strs) {
          vari = TypeVari::Path(vec![res_id]);
        } else {
          return Err(Message::new(MsgKind::Error, p[0].pos, "unknown type path".to_string(), path_strs));
        }
      }
      
      TypeVari::PointerOf{sub, ..} => {
        self.check_type(*sub)?;
      }
      
      TypeVari::ArrayOf{sub, ..} => {
        self.check_type(*sub)?;
      }
      
      TypeVari::ReferenceOf{sub, ..} => {
        let sub_state = self.mol.get_type(*sub).state;
        if sub_state != TypeState::Resolving {
          self.check_type(*sub)?;
        }
      }
      
      TypeVari::Function(f) => {
        for arg in &f.args {
          self.check_type(arg.kind)?;
        }
        self.check_type(f.ret)?;
      }
      
      TypeVari::Struct(s) => {
        for b in &s.base {
          self.check_type(*b)?;
        }
        for v in &s.vars {
          self.check_type(v.kind)?;
        }
        for f in &s.funs {
          self.check_decl(*f)?;
        }
      }
      
      TypeVari::Iface(i) => {
        for v in &i.funs {
          self.check_type(v.kind)?;
        }
      }
      
      TypeVari::Trait(i) => {
        for v in &i.funs {
          self.check_type(v.kind)?;
        }
      }
      
      TypeVari::Path(path) => {
        for p in path {
          self.check_type(*p)?;
        }
      }
      
      TypeVari::Enum(e) => {
        let mut names = std::collections::HashSet::new();
        for v in &e.vals {
          if !names.insert(v.name.str()) {
            return Err(Message::error(v.name, format!("enum variants must be unique, found duplicate `{}`", v.name.str()), vec![]));
          }
        }
      }

      TypeVari::Flags(e) => {
        let mut names = std::collections::HashSet::new();
        for v in &e.vals {
          if !names.insert(v.name.str()) {
            return Err(Message::error(v.name, format!("enum variants must be unique, found duplicate `{}`", v.name.str()), vec![]));
          }
        }
      }

      TypeVari::ArchSize{..} |
      TypeVari::Int{..} |
      TypeVari::Float{..} |
      TypeVari::Bool |
      TypeVari::Char |
      TypeVari::Ptr |
      TypeVari::Void |
      TypeVari::Null |
      TypeVari::SelfType => {}
      
      TypeVari::GenericInstance{base, args} => {
        self.check_type(*base)?;
        for arg in args.iter() {
          self.check_type(*arg)?;
        }
        
        let base_ty = self.mol.get_type(*base).vari.clone();
        if let TypeVari::Path(p) = base_ty {
          let base_decl_id = p[0];
          
          let inst_key = (base_decl_id, args.clone());
          
          if let Some(inst_decl_id) = self.mol.map_inst.get(&inst_key).copied() {
            let inst_decl = self.mol.get_decl(inst_decl_id);
            let type_id = if let crate::ast::DeclVari::Using(uid) = &inst_decl.vari {
              *uid
            } else {
              inst_decl_id
            };
            vari = TypeVari::Path(vec![type_id]);
          } else {
            let (params_clone, inner_decls) = {
              let base_decl = self.mol.get_decl(base_decl_id);
              if let crate::ast::DeclVari::Generic(g) = &base_decl.vari {
                (g.params.clone(), g.decls.clone())
              } else {
                let pos = crate::sema::type_s::get_word_from_type(self.mol, id).unwrap();
                return Err(Message::error(pos, "expected generic type".to_string(), vec![]));
              }
            };
            
            let mut sub_map = std::collections::HashMap::new();
            for (i, param) in params_clone.iter().enumerate() {
              let param_name = param.name.str().to_string();
              if i < args.len() {
                sub_map.insert(param_name, args[i]);
              }
            }
            
            let mut specializer = crate::sema::specialize::Specializer::new(self.mol, sub_map);
            
            let inst_decl_id = specializer.clone_decl(inner_decls[0]);
            let old_name = specializer.mol.get_decl(inst_decl_id).name.to_string();
            let mut new_name = format!("{}G{}", old_name, args.len());
            for arg in args.iter() {
              new_name.push_str(&crate::hgen::mangle::qw::QwMangler::mangle_type(*arg, specializer.mol));
            }
            {
              let inst_decl = specializer.mol.get_mut_decl(inst_decl_id);
              inst_decl.name = crate::ast::DeclName::Name(new_name);
            }

            let root_id = crate::control::identy::AstId::new(crate::control::identy::IdentyKind::Decl, 0, 0);
            if let crate::ast::DeclVari::Module(m) = &mut specializer.mol.get_mut_decl(root_id).vari {
              m.decls.push(inst_decl_id);
            }

            let inst_decl = specializer.mol.get_decl(inst_decl_id);
            let type_id = if let crate::ast::DeclVari::Using(uid) = &inst_decl.vari {
              *uid
            } else {
              inst_decl_id // Fallback, though shouldn't happen for structs
            };

            specializer.mol.map_inst.insert(inst_key, inst_decl_id);
            
            vari = TypeVari::Path(vec![type_id]);
            
            self.check_decl(inst_decl_id)?;
          }
        } else {
          let pos = crate::sema::type_s::get_word_from_type(self.mol, id).unwrap();
          return Err(Message::error(pos, "expected valid path for generic base".to_string(), vec![]));
        }
      }
    }

    let ty = self.mol.get_mut_type(id);
    ty.vari = vari;
    ty.state = TypeState::Resolved;

    Ok(())
  }

}


fn get_word_from_type<'a>(mol: &crate::control::Module<'a, '_>, id: AstId) -> Option<crate::lexer::Word<'a>> {
  let ty = mol.get_type(id);
  match &ty.vari {
    TypeVari::Nick(n) => Some(n.pos),
    TypeVari::PointerOf{sub, ..} | TypeVari::ReferenceOf { sub, .. } => get_word_from_type(mol, *sub),
    TypeVari::ArrayOf{sub, ..} => get_word_from_type(mol, *sub),
    TypeVari::Function(f) => get_word_from_type(mol, f.ret),
    _ => None
  }
}

impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn get_ty_void(&mut self) -> AstId {
    ScopeManager::find(self.mol, self.mol.get_mod(), &["sys".to_string(), "void".to_string()]).expect("void type missing")
  }
  
  pub fn get_ty_bool(&mut self) -> AstId {
    ScopeManager::find(self.mol, self.mol.get_mod(), &["sys".to_string(), "bool".to_string()]).expect("bool type missing")
  }

}
