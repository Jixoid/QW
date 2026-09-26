use crate::{Ident, PattRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Patt {
  One(Ident),
  
  /// _
  Under,
  /// ..
  Rest,
  
  Tuple(PattRng),
  Array(PattRng),
}
