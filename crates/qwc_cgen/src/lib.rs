use qwc_mir as mir;


pub trait ICGen {
  fn generate(&self, mir: &mir::Krate, ext_ll: bool) -> Result<(Vec<u8>, Option<String>), String>;

  fn run_vm(&self, code: &[u8]) -> Result<i32, String>;
}
