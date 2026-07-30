pub mod bare;
pub mod itanium;
pub mod qw;

use crate::hir::identy::HirId;
use crate::hir::module::HirModule;


pub trait Mangler {
  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<HirId>, ret_ty: Option<HirId>, arg_tys: &[HirId], hir_mol: &HirModule) -> String;
  fn mangle_global(&self, path: &[String], var_name: &str) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManglerKind {
  Itanium,
  Qw,
  Bare,
}

impl ManglerKind {
  pub fn get_mangler(&self) -> Box<dyn Mangler> {
    match self {
      ManglerKind::Itanium => Box::new(itanium::ItaniumMangler::new()),
      ManglerKind::Qw => Box::new(qw::QwMangler::new()),
      ManglerKind::Bare => Box::new(bare::BareMangler::new()),
    }
  }
}

impl Mangler for ManglerKind {
  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<HirId>, ret_ty: Option<HirId>, arg_tys: &[HirId], hir_mol: &HirModule) -> String {
    self.get_mangler().mangle_func(path, func_name, self_ty, ret_ty, arg_tys, hir_mol)
  }

  fn mangle_global(&self, path: &[String], var_name: &str) -> String {
    self.get_mangler().mangle_global(path, var_name)
  }
}
