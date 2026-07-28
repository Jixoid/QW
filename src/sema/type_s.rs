use crate::diagnostic::{Message, MsgKind};
use super::Sema;
use crate::control::identy::IdentyId;
use crate::ast::{TypeVari, TypeState};
use std::mem;

impl<'f, 'a, 'd> Sema<'f, 'a, 'd> {

  pub fn find_type_global(&mut self, name: &str) -> Option<IdentyId> {
    if let Some(id) = self.scp.find_type(name) {
      return Some(id);
    }
    
    // Search in dependencies
    for m in self.mol.imod.iter() {
      for decl in &m.list_decl {
        if let crate::ast::Visibility::Public = decl.vis {
          if let crate::ast::DeclVari::Using(ty_id) = decl.vari {
            if decl.name.to_string() == name {
              return Some(self.mol.localize(m, ty_id));
            }
          }
        }
      }
    }
    None
  }

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
        
        if let Some(res_id) = self.find_type_global(&name_str) {
          vari = TypeVari::Path { path: vec![res_id] };
        } else {
          return Err(Message::new(MsgKind::Error, s.pos, "unknown type".to_string(), vec![name_str]));
        }
      }
      TypeVari::PointerOf{sub, ..} | TypeVari::ZArrayOf(sub) => {
        self.check_type(*sub)?;
      }
      TypeVari::PArrayOf(s) => {
        self.check_type(s.sub)?;
      }
      TypeVari::ReferenceOf{sub, ..} => {
        self.check_type(*sub)?;
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
      }
      TypeVari::Path { path } => {
        for p in path {
          self.check_type(*p)?;
        }
      }
      _ => {}
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
    self.find_type_global("void").expect("void type missing")
  }
  
  pub fn get_ty_bool(&mut self) -> IdentyId {
    self.find_type_global("bool").expect("bool type missing")
  }

  pub fn find_var_global(&mut self, name: &str) -> Option<IdentyId> {
    if let Some(id) = self.scp.find_var(name) {
      return Some(id);
    }
    
    for m in self.mol.imod.iter() {
      for decl in &m.list_decl {
        if let crate::ast::Visibility::Public = decl.vis {
          if let crate::ast::DeclVari::Var(v) = &decl.vari {
            if decl.name.to_string() == name {
              return Some(self.mol.localize(m, v.kind));
            }
          }
        }
      }
    }
    None
  }

}
