use crate::{ast::{ExprId, Rng, TypeId}, lexer::Span};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Visibility {
  Public,
  Private,
  Protected,
  Crate,
  Super,
  Group,
}


#[derive(Debug)]
pub enum DeclVari {
  Var  {kind: TypeId, init: Option<ExprId>},
  Let  {kind: TypeId, init: Option<ExprId>},
  Fun  {kind: TypeId, blok: Option<ExprId>},
  Init {kind: TypeId, blok: Option<ExprId>, ils: Option<Rng> /* NamedTypeList */},
  Fini {kind: TypeId, blok: Option<ExprId>},
  Using{kind: TypeId},
}


pub struct Decl {
  pub name: Span,
  pub vari: DeclVari,
  pub vis: Visibility,
}
