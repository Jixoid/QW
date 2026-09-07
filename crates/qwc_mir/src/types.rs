use crate::{Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Type {
  // NST
  Unit,
  
  // Primitive
  Int(u32, bool),
  Float(FloatKind),
  
  Bool,

  // Combinated
  Struct(Rng),
  
  // Sequential
  Array(TypeId, u32),
  
  // Callable
  Fun{args: Rng, ret: TypeId},

  // Reference
  Ptr(TypeId)
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FloatKind {
  BF16,
  F16,
  F32,
  F64,
  F128,
}
