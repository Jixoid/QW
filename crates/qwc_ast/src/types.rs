use qwc_diagnostic::Span;

use crate::{ExprId, FieldRng, Ident, ThingRng, TypeId, TypeRng, AnyRng};



pub enum FunAttrs {
  Pure = 0x01,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Must Resolve
  Nick (Ident),
  Path (TypeRng),

  // Pointer
  Ptr (TypeId, bool),
  Ref (TypeId, bool),
  
  /// Vector
  Vector (TypeId, ExprId),
  VScale (TypeId),

  // Sequential
  Array (TypeId, ExprId),
  Slice (TypeId),
  
  // Must Context
  Type,
  SelfT,
  
  // Basic
  Unit,
  Range (TypeId),

  // Variant
  Fail    (TypeId),
  Option  (TypeId),
  Result  {sub: TypeId, err: TypeId},
  Variant (ThingRng /* Name | NamedType */),

  // Enum
  Enum  (ThingRng /* Name | NamedExpr */),
  Flags (ThingRng /* Name | NamedExpr */),

  // Combinated
  Struct (ThingRng /* NamedType => name: type */),
  Tuple  (TypeRng),

  // Impl
  Iface (FieldRng),
  Trait (FieldRng),

  // Function
  Fun {self_kind: Option<TypeId>, args: ThingRng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
  Task{self_kind: Option<TypeId>, args: ThingRng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
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
