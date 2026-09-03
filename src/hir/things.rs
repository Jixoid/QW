use crate::{hir::{Rng, TypeId}, lexer::Span};


pub enum Thing {
  Name(Span),
  List(Rng),

  NamedType(Span, TypeId),
}
