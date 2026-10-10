/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;
use qwc_string_interner::Sid;

use crate::{DefPathId, ExprId, ItemRng, TypeId, TypeRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymVis { Internal, Export, Import }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemVis { Private, Public(SymVis) }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
  RootNS {rng: ItemRng},
  NameSpace {rng: ItemRng},
  GenericNS {rng: ItemRng, args: TypeRng},

  Using {kind: TypeId},
  
  Variable {kind: TypeId, expr: ExprId, ism: bool},
  Function {kind: TypeId, expr: ExprId},
  Task {kind: TypeId, expr: ExprId},

  Impl {type_ty: TypeId, trait_ty: Option<TypeId>, methods: ItemRng},
}

bitflags! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub struct ItemAttr: u16 {
    const C = 1;
    const Entry = 2;
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
  pub vis: ItemVis,
  pub name: Option<Sid>,
  pub kind: ItemKind,
  pub attr: ItemAttr,
  pub path: DefPathId,
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
