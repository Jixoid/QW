use crate::ast::{self, Rng, Visibility};


#[derive(Debug)]
pub enum ItemVari {
  Module{name: String, ctn: Rng /* DeclId | ItemId */},
  
  Generic{params: Rng /* NamedType */, reqs: Rng /* NamedTypeList */, ctn: Rng /* DeclId | ItemId */},
  
  Impl{type_ty: ast::TypeId, trait_ty: Option<ast::TypeId>, ctn: Rng /* DeclId */},

  Import(Rng /* ThingId */),
}


pub struct Item {
  pub vari: ItemVari,
  pub vis: Visibility,
}
