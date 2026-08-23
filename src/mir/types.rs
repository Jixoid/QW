use std::num::NonZeroUsize;

use crate::mir::TypeId;


pub enum Type {
  Int(u16, bool),
  Float(u16),
  Bool,
  Char,
  Ptr,
  Unit,

  Array(TypeId, Option<NonZeroUsize>),

  Function(Vec<TypeId>, TypeId),

  Struct(Vec<TypeId>),
}
