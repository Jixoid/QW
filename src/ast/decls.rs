use crate::{ast::{AccessKind, ExprId, TypeId}, lexer::Word};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Visibility {
  Public,
  Private,
  Protected,
  Crate,
  Group,
}


pub enum DeclVari {
  Var  {kind: TypeId, acck: AccessKind, init: Option<ExprId>},
  Fun  {kind: TypeId, blok: ExprId},
  Using{kind: TypeId},
}


pub struct Decl {
  pub name: Word,
  pub vari: DeclVari,
  pub vis: Visibility,
}
