use std::path::Path;

use qwc_mir as mir;


pub trait ICGen {
  fn generate(mir: &mir::Krate, fpath: &Path) -> Result<(), String>;
}
