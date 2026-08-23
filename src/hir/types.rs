use crate::hir::{Rng, TypeId};


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }


pub enum Type {
  Ptr(TypeId, AccessKind),
  Ref(TypeId, AccessKind),
  DynArr(TypeId),
  StaArr(TypeId, usize),

  Range(TypeId),
  Option(TypeId),
  Result(TypeId, TypeId),
  Vector(TypeId, usize),

  Unit, Bool, Char, Str,

  MetaType,
  
  Int(u16, bool), ArchInt(bool),
  Float(u16), ArchFloat,

  Struct(Rng /* NamedTypeVis */),
  Tuple (Rng /* TypeId */),

  Iface (Rng /* NamedTypeVis */),
  Trait (Rng /* NamedTypeVis */),

  Enum (Rng /* NamedExpr */),
  Flags(Rng /* NamedExpr */),

  Fun{args: Rng /* NamedType */, ret: Option<TypeId>},
}
