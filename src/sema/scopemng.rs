use std::collections::HashMap;
use crate::control::identy::IdentyId;


pub struct Scope {
  pub types: HashMap<String, IdentyId>,
  pub vars: HashMap<String, IdentyId>,
}

impl Scope {

  pub fn new() -> Self {
    Self {
      types: HashMap::new(),
      vars: HashMap::new(),
    }
  }

}

pub struct ScopeManager {
  scopes: Vec<Scope>,
}

impl ScopeManager {
  
  pub fn new() -> Self {
    Self{ scopes: vec![Scope::new()] }
  }

  
  pub fn push(&mut self) {
    self.scopes.push(Scope::new());
  }

  pub fn pop(&mut self) {
    self.scopes.pop();
  }

  
  pub fn add_type(&mut self, name: String, id: IdentyId) {
    self.scopes.last_mut().unwrap().types.insert(name, id);
  }

  pub fn find_type(&self, name: &str) -> Option<IdentyId> {
    for scope in self.scopes.iter().rev() {
      if let Some(&id) = scope.types.get(name) {
        return Some(id);
      }
    }
    None
  }


  pub fn add_var(&mut self, name: String, id: IdentyId) {
    self.scopes.last_mut().unwrap().vars.insert(name, id);
  }

  pub fn find_var(&self, name: &str) -> Option<IdentyId> {
    for scope in self.scopes.iter().rev() {
      if let Some(&id) = scope.vars.get(name) {
        return Some(id);
      }
    }
    None
  }

}
