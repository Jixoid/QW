use crate::{ast::{self, Rng, Visibility}, lexer::Span};


pub enum ItemVari {
  Module{name: String, ctn: Rng},
  
  Generic{params: Vec<ast::FieldType>, ctn: Rng, reqs: Rng},
  
  Impl{trait_ty: ast::TypeId, type_ty: ast::TypeId, ctn: Vec<ast::DeclId>},

  Import(Vec<(u32, Span)>, Option<ast::DeclId>),
  ImportWildcard(Vec<(u32, Span)>, Option<ast::DeclId>),
}

pub struct Item {
  pub vari: ItemVari,
  pub vis: Visibility,
}
