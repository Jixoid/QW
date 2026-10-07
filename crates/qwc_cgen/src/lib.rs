use qwc_mir as mir;


pub enum Optimization {
  None = 0,
  Less = 1,
  Default = 2,
  Aggressive = 3,
}

pub enum OutKind {
  Object, ByteCode
}


pub trait ICGen {
  fn generate(&self, mir: &mir::Krate, ext_ll: bool, outk: OutKind, triple: &Option<String>, opt: Optimization) -> Result<(Vec<u8>, Option<String>), String>;

  fn run_vm(&self, code: &[u8]) -> Result<i32, String>;
}
