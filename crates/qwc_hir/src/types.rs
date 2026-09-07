use crate::{ExprId, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Type {
  // Virtual Generic
  GenericType,

  // NST
  Unit,

  // Primitive
  Int(u16, bool),
  ArchInt(bool),
  Float(u16),

  Bool,

  // Combinated
  Struct(Rng /* TypeId */),

  // Sequential
  Array(TypeId, ExprId),

  // Callable
  Fun{args: Rng /* TypeId */, ret: TypeId},
  
  // Reference
  Ref(TypeId),
  Ptr(TypeId),
  Slice(TypeId),
}
