use bitflags::bitflags;
use qwc_string_interner::Sid;

use crate::{ExprId, ItemRng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymVis { Internal, Export, Import }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemVis { Private, Public(SymVis) }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
  RootNS {rng: ItemRng},
  NameSpace {rng: ItemRng, name: Sid},
  GenericNS {rng: ItemRng},

  Using {kind: TypeId, name: Sid},
  
  Variable {kind: TypeId, expr: ExprId, name: Sid, ism: bool},
  Function {kind: TypeId, expr: ExprId, name: Sid},

  Impl {type_ty: TypeId, trait_ty: Option<TypeId>, methods: ItemRng},
}

bitflags! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub struct ItemAttr: u16 {
    const Entry = 1;
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
  pub vis: ItemVis,
  pub kind: ItemKind,
  pub attr: ItemAttr,
}

impl Item {
  pub fn is_symbol(&self) -> bool {
    matches!(self.kind, ItemKind::Function{..} | ItemKind::Variable{..})
  }
  
  pub fn symbol_kind(&self) -> Option<TypeId> {
    match self.kind {
      ItemKind::Function{kind, ..} | ItemKind::Variable{kind, ..} => Some(kind),
      _ => None
    }
  }

  pub fn assignable(&self) -> Option<bool> {
    match self.kind {
      ItemKind::Function{..} => Some(false),
      ItemKind::Variable{ism, ..} => Some(ism),

      _ => None,
    }
  }
}
