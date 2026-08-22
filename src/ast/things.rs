use crate::{ast::{ExprId, Rng, TypeId, Visibility}, lexer::Span};


#[derive(Debug)]
pub enum Thing {
  Name(Span),
  List(Rng),

  /// *
  Wildcard,

  Crate,
  Super,
  
  NamedExpr(Span, ExprId),
  NamedType(Span, TypeId),
  
  NamedTypeVis(Span, Visibility, TypeId),

  NamedTypeList(Span, Rng),
}
