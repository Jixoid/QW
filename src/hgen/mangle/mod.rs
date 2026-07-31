pub mod bare;
pub mod itanium;
pub mod qw;

use crate::control::identy::IdentyId;
use crate::control::module::Module;

pub trait Mangler {
  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<IdentyId>, ret_ty: Option<IdentyId>, arg_tys: &[IdentyId], ast_mol: &Module) -> String;
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
  fn mangle_func(&self, path: &[String], func_name: &str, self_ty: Option<IdentyId>, ret_ty: Option<IdentyId>, arg_tys: &[IdentyId], ast_mol: &Module) -> String {
    self.get_mangler().mangle_func(path, func_name, self_ty, ret_ty, arg_tys, ast_mol)
  }

  fn mangle_global(&self, path: &[String], var_name: &str) -> String {
    self.get_mangler().mangle_global(path, var_name)
  }
}
