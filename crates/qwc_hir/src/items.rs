/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;
use qwc_string_interner::Sid;

use qwc_serial::{Decoder, Encoder};
use qwc_serial_macro::{Decode, Encode};

use crate::{DefPathId, ExprId, ItemRng, PathApi, ThingRng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum SymVis { Internal, Export, Import }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum ItemVis { Private, Public(SymVis) }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum ItemKind {
  RootNS {rng: ItemRng},
  NameSpace {rng: ItemRng},
  GenericNS {rng: ItemRng, args: ThingRng},

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

impl Encoder for ItemAttr {
  fn encode(&self, ctx: &mut qwc_serial::EncodeCtx) -> Result<(), qwc_serial::EncodeErr> {
    self.bits().encode(ctx)
  }
}

impl Decoder for ItemAttr {
  fn decode(ctx: &mut qwc_serial::DecodeCtx) -> Result<Self, qwc_serial::DecodeErr> {
    let bits = u16::decode(ctx)?;
    Ok(Self::from_bits_truncate(bits))
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Item {
  pub vis: ItemVis,
  pub name: Option<Sid>,
  pub kind: ItemKind,
  pub attr: ItemAttr,
  pub path: DefPathId,
}


impl PathApi for Item {
  fn path(&self) -> DefPathId {
    self.path
  }
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
