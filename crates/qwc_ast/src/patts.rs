use crate::{Ident, Rng};


#[derive(Debug, Copy, Clone)]
pub enum Patt {
  One(Ident),
  
  /// _
  Under,
  /// ..
  Rest,
  
  Tuple(Rng),
  Array(Rng),
}
