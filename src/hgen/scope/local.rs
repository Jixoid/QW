use std::num::NonZeroU32;

use crate::hir;


pub struct Local {
  scopes: Vec<std::collections::HashMap<NonZeroU32, (hir::TypeId, u32)>>,
  next_idx: u32,
}


impl Local {

  pub fn new() -> Self {
    Self{
      scopes: vec![],
      next_idx: 0,
    }
  }


  pub fn enter(&mut self) {
    self.scopes.push(std::collections::HashMap::new());
  }

  pub fn leave(&mut self) {
    self.scopes.pop();
  }

  
  pub fn lookup(&self, sid: NonZeroU32) -> Option<(hir::TypeId, u32)> {
    for scp in self.scopes.iter().rev() {
      if let Some(&found) = scp.get(&sid) {
        return Some(found);
      }
    }
    None
  }


  pub fn bind(&mut self, sid: NonZeroU32, ty: hir::TypeId) -> u32 {
    let idx = self.next_idx;
    self.next_idx += 1;

    if let Some(cur) = self.scopes.last_mut() {
      cur.insert(sid, (ty, idx));
    }
    idx
  }

}
