use std::ptr;

use crate::hir::{self, HirId, HirKind};


pub struct Crate<'a> {
  pub imod: Vec<&'a Crate<'a>>,

  pub list_type: Vec<hir::Type>,
  pub list_decl: Vec<hir::Decl>,
}


impl<'a> Crate<'a> {

  pub fn new() -> Self {
    Self{
      imod: vec![],

      list_type: vec![],
      list_decl: vec![],
    }
  }


  pub fn add_dep(&mut self, mol: &'a Self) -> u16 {
    match self.imod.iter().position(|&x| ptr::eq(x, mol)) {
      Some(r) => (r as u16) +1,
      None => {
        self.imod.push(mol);
        self.imod.len() as u16
      }
    }
  }

  pub fn get_dep(&self, mod_id: u16) -> &'a Self {
    self.imod[(mod_id -1) as usize] 
  }


  pub fn localize<T>(&mut self, mol: &'a Self, eid: HirId<T>) -> HirId<T> {
    let omol = if eid.krate == 0 { mol } else { mol.get_dep(eid.krate) };

    let lmid = self.add_dep(omol);

    HirId::new(eid.kind, lmid, eid.index)
  }


  pub fn get_type(&self, id: hir::TypeId) -> &hir::Type {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_type[id.index as usize]
  }


  pub fn new_type(&mut self, v: hir::Type) -> hir::TypeId {
    self.list_type.push(v);

    let idx = self.list_type.len() as u32 - 1;

    return HirId::new(HirKind::Type, 0, idx);
  }


  pub fn storage_size(&self) -> usize {
    size_of::<hir::Type>()*self.list_type.len()
    +
    size_of::<hir::Decl>()*self.list_decl.len()
  }

}
