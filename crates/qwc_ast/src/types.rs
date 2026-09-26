use qwc_diagnostic::Span;

use crate::{ExprId, FieldRng, Ident, ThingRng, TypeId, TypeRng, AnyRng};



pub enum FunAttrs {
  Static = 0x01,
  Const  = 0x02,
  Pure   = 0x04,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Access
  Nick (Ident),
  Path (TypeRng),

  // Pointer
  Ptr (TypeId, bool),
  Ref (TypeId, bool),
  
  // Heap
  Array  (TypeId, ExprId),
  Slice  (TypeId),
  Vector (TypeId, ExprId),
  
  // Basic
  Unit,
  Range  (TypeId),
  Option (TypeId),
  Fail   (TypeId),

  // Intrinsic
  Type (),
  SelfT(),

  // Variant
  Result  {sub: TypeId, err: TypeId},
  Variant (ThingRng /* Name | NamedType */),

  // Enum
  Enum  (ThingRng /* Name | NamedExpr */),
  Flags (ThingRng /* Name | NamedExpr */),

  // Data
  Struct (FieldRng),
  Tuple  (TypeRng),

  // Impl
  Iface (FieldRng),
  Trait (FieldRng),

  // Function
  Fun {args: ThingRng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
  Init{args: ThingRng /* NamedType */, attr: u8 /* FunAttrs */},
  Fini{args: ThingRng /* NamedType */, attr: u8 /* FunAttrs */},

  // Specialize
  Spec{base: TypeId, args: AnyRng /* TypeId | ExprId */},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub pos: Span,
  pub kind: TypeKind,
}
