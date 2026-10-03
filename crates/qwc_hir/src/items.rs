use qwc_string_interner::Sid;

use crate::{ExprId, ItemRng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymVis { Export, Import }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemVis { Private, Public }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
  RootNS {rng: ItemRng},
  NameSpace {rng: ItemRng, name: Sid},
  GenericNS {rng: ItemRng},

  Using {kind: TypeId, name: Sid},
  
  Variable {kind: TypeId, expr: ExprId, name: Sid, ism: bool},
  Function {kind: TypeId, expr: ExprId, name: Sid},

  Impl {struct_ty: TypeId, iface_ty: Option<TypeId>, methods: ItemRng},
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
