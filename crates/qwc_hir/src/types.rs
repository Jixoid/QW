use crate::{ExprId, Layout, ThingRng, TypeId, TypeRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Generic
  GenericType,
  GenericSelfType,
  
  // Basic
  Unit,
  Never,
  
  // Primitive
  Int(u16, bool),
  ArchInt(bool),
  Float(u16),
  Bit(u16),
  Bool,

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
  Trait(ThingRng /* NamedType => fun name: type */),
  Iface(ThingRng /* NamedType => fun name: type */),

  TraitFrom{trait_ty: TypeId, hidden: TypeId},

  // Variant
  Option(TypeId),
  
  // Callable
  Fun{self_kind: Option<TypeId>, args: ThingRng /* NamedType => name: type */, ret: TypeId},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub kind: TypeKind,
  pub layout: Layout,
}
