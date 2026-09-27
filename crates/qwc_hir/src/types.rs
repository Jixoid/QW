use crate::{ExprId, Layout, TypeId, TypeRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Virtual Generic
  GenericType,
  
  // ZST
  Unit,
  
  // Never
  Never,
  
  // Primitive
  Bit(u16),
  Int(u16, bool),
  ArchInt(bool),
  Float(u16),
  
  Bool,

  // Combinated
  Struct(TypeRng),
  
  // Sequential
  Array(TypeId, ExprId),
  
  // Callable
  Fun{args: TypeRng, ret: TypeId},
  
  // Reference
  Ref(TypeId, bool /* ism */),
  Ptr(TypeId, bool /* ism */),
  Slice(TypeId),

  // Option
  Option(TypeId),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub kind: TypeKind,
  pub layout: Layout,
}
