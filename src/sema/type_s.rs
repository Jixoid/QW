use crate::diagnostic::{Message, MsgKind};
use crate::sema::scopemng::ScopeManager;
use super::Sema;
use crate::control::identy::IdentyId;
use crate::ast::{TypeVari, TypeState};
use std::mem;


impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn check_type(&mut self, id: IdentyId) -> Result<(), Message<'a>> {
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
      
      TypeVari::PointerOf{sub, ..} | TypeVari::ZArrayOf(sub) => {
        self.check_type(*sub)?;
      }
      
      TypeVari::PArrayOf(s) => {
        self.check_type(s.sub)?;
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
    }

    let ty = self.mol.get_mut_type(id);
    ty.vari = vari;
    ty.state = TypeState::Resolved;

    Ok(())
  }

}


fn get_word_from_type<'a>(mol: &crate::control::Module<'a, '_>, id: IdentyId) -> Option<crate::lexer::Word<'a>> {
  let ty = mol.get_type(id);
  match &ty.vari {
    TypeVari::Nick(n) => Some(n.pos),
    TypeVari::PointerOf{sub, ..} | TypeVari::ZArrayOf(sub) | TypeVari::ReferenceOf { sub, .. } => get_word_from_type(mol, *sub),
    TypeVari::PArrayOf(s) => get_word_from_type(mol, s.sub),
    TypeVari::Function(f) => get_word_from_type(mol, f.ret),
    _ => None
  }
}

impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn get_ty_void(&mut self) -> IdentyId {
    ScopeManager::find(self.mol, self.mol.get_mod(), &["sys".to_string(), "void".to_string()]).expect("void type missing")
  }
  
  pub fn get_ty_bool(&mut self) -> IdentyId {
    ScopeManager::find(self.mol, self.mol.get_mod(), &["sys".to_string(), "bool".to_string()]).expect("bool type missing")
  }

}
