use crate::hir::module::HirModule;

pub trait ICGen {
  fn generate(&mut self, hir_mol: &HirModule) -> String;
}
