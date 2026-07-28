use crate::hir::module::HirModule;
use super::icgen::ICGen;


pub struct CGen<'a, B: ICGen> {
	hir_mol: &'a HirModule,
	backend: B,
}

impl<'a, B: ICGen> CGen<'a, B> {
  pub fn new(hir_mol: &'a HirModule, backend: B) -> Self {
    Self {hir_mol, backend}
  }

  pub fn generate(&mut self) -> String {
    self.backend.generate(self.hir_mol)
  }
}
