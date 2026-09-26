use crate::{ExprId, Layout, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Virtual Generic
  GenericType,
  
  // ZST
  Unit,
  
  // Never
  Never,
  
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
