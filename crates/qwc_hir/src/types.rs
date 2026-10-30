/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;
use qwc_serial::{Decoder, Encoder};
use qwc_serial_macro::{Decode, Encode};

use crate::{DefPathId, ExprId, ItemId, Layout, ThingRng, TypeId, TypeRng};


#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode)]
pub enum TypeKind {
  // Generic
  GenericType{generic: ItemId, idx: u32},
  GenericRaw{generic: ItemId, kind: TypeId},
  GenericSelfType,
  AssociatedType{trait_ty: TypeId, idx: u32},
  
  // Basic
  Error,
  Unit,
  Never,
  
  // Primitive
  Int(u16, bool),
  ArchInt(bool),
  Float(u16),
  Bit(u16),
  Bool,
  Char,
  
  Str,

  // Meta
  Meta(TypeId),
  
  // Reference
  Ref {kind: TypeId, ism: bool, addrspace: Option<ExprId>},
  Ptr {kind: TypeId, ism: bool, addrspace: Option<ExprId>},
  
  // VScale
  Vector(TypeId, ExprId),
  VScale(TypeId),

  // Sequential
  Array(TypeId, ExprId),
  Slice(TypeId),
  
  // Combinated
  Struct(ThingRng /* NamedType => name: type */),
  Tuple(TypeRng),
  
  ItemStruct{fields: ThingRng /* NamedType => name: type */, path: DefPathId},
  ItemTuple{fields: TypeRng, path: DefPathId},
  
  Range(TypeId),
  
  // Trait
  ItemTrait{methods: ThingRng /* NamedType => fun name: type */, types: ThingRng /* Name | NamedType => using (= type)?; */, reqs: TypeRng, path: DefPathId},
  ItemIface{methods: ThingRng /* NamedType => fun name: type */, path: DefPathId},
  
  TraitFrom{trait_ty: TypeId, hidden: TypeId},
  
  // Enum
  ItemEnum{values: ThingRng /* NamedExpr => name = expr */, path: DefPathId},
  ItemFlag{values: ThingRng /* NamedExpr => name = expr */, path: DefPathId},
  
  // Variant
  Option(TypeId),
  
  // Callable
  Fun{self_kind: Option<TypeId>, args: ThingRng /* NamedType => name: type */, ret: TypeId},
}


impl TypeKind {
  pub fn path(&self) -> Option<DefPathId> {
    match *self {
      TypeKind::ItemStruct { path, .. }
      | TypeKind::ItemTuple { path, .. }
      | TypeKind::ItemTrait { path, .. }
      | TypeKind::ItemIface { path, .. }
      | TypeKind::ItemEnum { path, .. }
      | TypeKind::ItemFlag { path, .. } => Some(path),
      _ => None,
    }
  }
}

bitflags! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub struct TypeAttr: u16 {
    const C = 1;
  }
}

impl Encoder for TypeAttr {
  fn encode(&self, ctx: &mut qwc_serial::EncodeCtx) -> Result<(), qwc_serial::EncodeErr> {
    self.bits().encode(ctx)
  }
}

impl Decoder for TypeAttr {
  fn decode(ctx: &mut qwc_serial::DecodeCtx) -> Result<Self, qwc_serial::DecodeErr> {
    let bits = u16::decode(ctx)?;
    Ok(Self::from_bits_truncate(bits))
  }
}



#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode)]
pub struct Type {
  pub kind: TypeKind,
  pub layout: Layout,
  pub attr: TypeAttr,
}

impl Type {
  pub fn path(&self) -> Option<DefPathId> {
    self.kind.path()
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PrimTypes {
  pub ty_type: TypeId,
  pub ty_generic_self_type: TypeId,
  pub ty_error: TypeId,
  pub ty_unit: TypeId,
  pub ty_never: TypeId,
  pub ty_bool: TypeId,
  pub ty_char: TypeId,
  pub ty_str: TypeId,
  
  pub ty_isize: TypeId,
  pub ty_usize: TypeId,

  pub ty_i8: TypeId,
  pub ty_i16: TypeId,
  pub ty_i32: TypeId,
  pub ty_i64: TypeId,
  pub ty_i128: TypeId,

  pub ty_u8: TypeId,
  pub ty_u16: TypeId,
  pub ty_u32: TypeId,
  pub ty_u64: TypeId,
  pub ty_u128: TypeId,
  
  pub ty_b8: TypeId,
  pub ty_b16: TypeId,
  pub ty_b32: TypeId,
  pub ty_b64: TypeId,
  pub ty_b128: TypeId,

  pub ty_f16: TypeId,
  pub ty_f32: TypeId,
  pub ty_f64: TypeId,
  pub ty_f128: TypeId,
}

#[derive(Debug, Copy, Clone)]
pub struct PrimOps {
  pub op_add: TypeId,
  pub op_sub: TypeId,
  pub op_mul: TypeId,
  pub op_div: TypeId,
  pub op_rem: TypeId,

  pub op_add_assign: TypeId,
  pub op_sub_assign: TypeId,
  pub op_mul_assign: TypeId,
  pub op_div_assign: TypeId,
  pub op_rem_assign: TypeId,

  pub op_neg: TypeId,
  pub op_not: TypeId,
  pub op_deref: TypeId,
  
  pub op_partial_eq: TypeId,
  pub op_eq: TypeId,
  pub op_partial_ord: TypeId,
  pub op_ord: TypeId,
  pub ty_ordering: TypeId,
}
