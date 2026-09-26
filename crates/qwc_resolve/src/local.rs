use qwc_diagnostic::Span;
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;
use qwc_hir as hir;


#[derive(Clone, Debug)]
pub struct LocalVarInfo {
  pub name: Sid,
  pub ty: hir::TypeId,
  pub ism: bool,
  pub span: Span,
}


#[derive(Clone, Debug)]
pub struct LocalScopeManager {
  pub locals: Vec<LocalVarInfo>,
  pub scopes: Vec<FxHashMap<Sid, u32>>,
}


impl LocalScopeManager {

  pub fn new() -> Self {
    Self {
      locals: Vec::new(),
      scopes: vec![FxHashMap::default()],
    }
  }

  pub fn push_scope(&mut self) {
    self.scopes.push(FxHashMap::default());
  }

  pub fn pop_scope(&mut self) {
    self.scopes.pop();
  }

  pub fn insert(&mut self, name: Sid, ty: hir::TypeId, ism: bool, span: Span) -> u32 {
    let id = self.locals.len() as u32;
    self.locals.push(LocalVarInfo { name, ty, ism, span });
    self.scopes.last_mut().expect("no active scope").insert(name, id);
    id
  }

  pub fn lookup(&self, name: &Sid) -> Option<u32> {
    for scope in self.scopes.iter().rev() {
      if let Some(&id) = scope.get(name) {
        return Some(id);
      }
    }
    None
  }

  pub fn get_local(&self, id: u32) -> &LocalVarInfo {
    &self.locals[id as usize]
  }

  pub fn locals_len(&self) -> usize {
    self.locals.len()
  }

}
