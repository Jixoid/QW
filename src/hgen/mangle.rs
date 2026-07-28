pub trait Mangler {
  fn mangle_func(&self, module_name: &str, func_name: &str) -> String;
  fn mangle_global(&self, module_name: &str, var_name: &str) -> String;
}

pub struct ItaniumMangler;

impl ItaniumMangler {
  pub fn new() -> Self {
    Self {}
  }
}

impl Mangler for ItaniumMangler {

  fn mangle_func(&self, module_name: &str, func_name: &str) -> String {

    format!("_ZN{}{}{}{}E", module_name.len(), module_name, func_name.len(), func_name)
  }

  fn mangle_global(&self, module_name: &str, var_name: &str) -> String {

    format!("_ZN{}{}{}{}E", module_name.len(), module_name, var_name.len(), var_name)
  }
  
}
