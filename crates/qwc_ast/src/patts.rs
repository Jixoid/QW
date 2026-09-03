use crate::{Ident, Rng};


#[derive(Copy, Clone)]
pub enum Patt {
  One(Ident),
  
  /// _
  Under,
  /// ..
  Rest,
  
  Tuple(Rng),
  Array(Rng),
}
