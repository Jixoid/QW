use qwc_diagnostic::Span;

use crate::{ExprId, Ident, Rng, TypeId};



pub enum FunAttrs {
  Static = 0x01,
  Const  = 0x02,
  Pure   = 0x04,
}


#[derive(Copy, Clone)]
pub enum TypeKind {
  // Access
  Nick (Ident),
  Path (Rng /* TypeId */),

  // Pointer
  Ptr (TypeId, bool),
  Ref (TypeId, bool),
  LVa (TypeId, bool),
  
  // Heap
  Array  (TypeId, ExprId),
  Slice  (TypeId),
  Vector (TypeId, ExprId),
  
  // Basic
  Unit   (),
  Range  (TypeId),
  Option (TypeId),
  Fail   (TypeId),

  // Variant
  Result  {sub: TypeId, err: TypeId},
  Variant (Rng /* Name | NamedType */),

  // Enum
  Enum  (Rng /* Name | NamedExpr */),
  Flags (Rng /* Name | NamedExpr */),

  // Data
  Struct (Rng /* ItemId */),
  Tuple  (Rng /* TypeVis */),

  // Impl
  Iface (Rng /* ItemId */),
  Trait (Rng /* ItemId */),

  // Function
  Fun {args: Rng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
  Init{args: Rng /* NamedType */, attr: u8 /* FunAttrs */},
  Fini{args: Rng /* NamedType */, attr: u8 /* FunAttrs */},

  // Specialize
  Spec{base: TypeId, args: Rng /* TypeId | ExprId */},
}


#[derive(Copy, Clone)]
pub struct Type {
  pub pos: Span,
  pub kind: TypeKind,
}
