use std::num::NonZeroU32;

use crate::{Layout, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // NST
  Unit,
  
  // Primitive
  Int(NonZeroU32, bool),
  Float(FloatKind),
  
  Bool,
  Ptr,

  // Combinated
  Struct(Rng /* TypeId */),
  
  // Sequential
  Array(TypeId, u32),
  
  // Callable
  Fun{args: Rng /* TypeId */, ret: TypeId},
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub kind: TypeKind,
  pub layout: Layout,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FloatKind {
  BF16,
  F16,
  F32,
  F64,
  F128,
}
