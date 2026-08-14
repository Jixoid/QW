use crate::{ast, lexer::Span};


#[derive(Clone)]
pub enum DirceVari {
  One(Span),
  Use(Span),
  Feat(Span),
  Set{key: Span, val: ast::ExprId},
  Cfg(ast::ExprId),
}

#[derive(Clone)]
pub struct Dirc {
  pub vari: DirceVari,
}
