use qwc_string_interner::Sid;

use crate::{ExprId, Rng, TypeId};


#[derive(Debug, Copy, Clone)]
pub enum SymVis {
  Export,
  Import,
}

#[derive(Debug, Copy, Clone)]
pub enum ItemVis {
  Private,
  Public,
}


#[derive(Debug, Copy, Clone)]
pub enum ItemKind {
  RootNS{rng: Rng /* ItemId */},
  NameSpace{rng: Rng /* ItemId */, name: Sid},
  GenericNS{rng: Rng /* ItemId */},
  
  Variable{kind: TypeId, expr: ExprId, name: Sid, ism: bool},
  Function{kind: TypeId, expr: ExprId, name: Sid},
}


#[derive(Debug, Copy, Clone)]
pub struct Item {
  pub vis: ItemVis,
  pub svis: Option<SymVis>,
  pub kind: ItemKind,
}
