use super::Mangler;


pub struct BareMangler;

impl BareMangler {
  pub fn new() -> Self { Self {} }
}

use crate::control::identy::IdentyId;
use crate::control::module::Module;

impl Mangler for BareMangler {
  fn mangle_func(&self, _path: &[String], func_name: &str, _self_ty: Option<IdentyId>, _ret_ty: Option<IdentyId>, _arg_tys: &[IdentyId], _ast_mol: &Module) -> String {
    func_name.to_string()
  }

  fn mangle_global(&self, _path: &[String], var_name: &str) -> String {
    var_name.to_string()
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bare_mangler() {
    let mangler = BareMangler::new();
    let path = vec!["std".to_string(), "io".to_string()];
    let sym_global = mangler.mangle_global(&path, "print");
    assert_eq!(sym_global, "print");
  }
}

