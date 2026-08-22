use crate::{ast::Rng, lexer::Span};


pub enum Patt {
  One(Span),
  
  /// _
  Under,
  /// ..
  Rest,
  
  Tuple(Rng),
  Array(Rng),
}
