use crate::{ast::Rng, lexer::Span};


pub enum Patt {
  One(Span),
  
  Wildcard,
  Rest,
  
  Tuple(Rng),
  Array(Rng),
}
