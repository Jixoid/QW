use crate::{Ident, Rng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Patt {
  One(Ident),
  
  /// _
  Under,
  /// ..
  Rest,
  
  Tuple(Rng),
  Array(Rng),
}
