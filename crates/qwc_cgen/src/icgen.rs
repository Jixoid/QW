use qwc_mir as mir;


pub trait ICGen {
  fn generate(mir: &mir::Krate);
}
