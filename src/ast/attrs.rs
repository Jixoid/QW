use crate::{ast::{self, Rng}, lexer::Span};


#[derive(Clone)]
pub enum AttrVari {
  One(Span),
  Bin(Span, Span),
  Set(Span, ast::ExprId), 
  List(Span, Rng),
}


#[derive(Clone)]
pub struct Attr {
  pub vari: AttrVari,
}
