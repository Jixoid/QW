use qwc_string_interner::Sid;

use crate::{ExprId, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymVis { Export, Import }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemVis { Private, Public }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
  RootNS{rng: Rng /* ItemId */},
  NameSpace{rng: Rng /* ItemId */, name: Sid},
  GenericNS{rng: Rng /* ItemId */},
  
  Variable{kind: TypeId, expr: ExprId, name: Sid, ism: bool},
  Function{kind: TypeId, expr: ExprId, name: Sid},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
  pub vis: ItemVis,
  pub svis: Option<SymVis>,
  pub kind: ItemKind,
}

impl Item {
  pub fn is_symbol(&self) -> bool {
    matches!(self.kind, ItemKind::Function{..} | ItemKind::Variable{..})
  }
  
  pub fn symbol_kind(&self) -> Option<TypeId> {
    match self.kind { ItemKind::Function{kind, ..} | ItemKind::Variable{kind, ..} => Some(kind), _ => None }
  }
}
