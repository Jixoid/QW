/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use bitflags::bitflags;

use crate::{ExprId, ItemId, Layout, ThingRng, TypeId, TypeRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Generic
  GenericType{idx: usize},
  GenericRaw{generic: ItemId, kind: TypeId},
  GenericSelfType,
  
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
  Str,

  // Meta
  Meta(TypeId),
  
  // Reference
  Ref(TypeId, bool /* ism */),
  Ptr(TypeId, bool /* ism */),
  
  // VScale
  Vector(TypeId, ExprId),
  VScale(TypeId),

  // Sequential
  Array(TypeId, ExprId),
  Slice(TypeId),
  
  // Combinated
  Struct(ThingRng /* NamedType => name: type */),
  Tuple(TypeRng),
  
  // Trait
  Trait(ThingRng /* NamedType => fun name: type */, ThingRng /* Name | NamedType => using (= type)?; */),
  Iface(ThingRng /* NamedType => fun name: type */),
  
  TraitFrom{trait_ty: TypeId, hidden: TypeId},
  
  // Enum
  Enum(ThingRng /* NamedExpr => name = expr */),  

  // Variant
  Option(TypeId),
  
  // Callable
  Fun{self_kind: Option<TypeId>, args: ThingRng /* NamedType => name: type */, ret: TypeId},
}


bitflags! {
  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub struct TypeAttr: u16 {
    const C = 1;
  }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub kind: TypeKind,
  pub layout: Layout,
  pub attr: TypeAttr,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PrimTypes {
  pub ty_type: TypeId,
  pub ty_generic_self_type: TypeId,
  pub ty_error: TypeId,
  pub ty_unit: TypeId,
  pub ty_never: TypeId,
  pub ty_bool: TypeId,
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
