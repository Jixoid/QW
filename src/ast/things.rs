use crate::{ast::{ExprId, Rng, TypeId}, lexer::Span};


pub enum Thing {
  Name(Span),
  
  NamedExpr(Span, ExprId),
  NamedType(Span, TypeId),

  NamedTypeList(Span, Rng),
}
