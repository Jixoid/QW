use crate::{ast::{ExprId, Rng, TypeId}, lexer::Span};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }


#[derive(Debug)]
pub enum FunAttrs {
  Static = 0x01,
  Const  = 0x02,
  Pure   = 0x04,
}


#[derive(Debug)]
pub enum Type {
  Nick{pos: Span, idx: u32},
  Path(Rng),

  Ptr   {sub: TypeId, acc: AccessKind},
  Ref   {sub: TypeId, acc: AccessKind},
  Array {sub: TypeId, ext: Option<Rng> /* ExprId */},
  Vector{sub: TypeId, ext: ExprId},
  Range {sub: TypeId},
  Option{sub: TypeId},
  Result{sub: TypeId, err: TypeId},
  
  Struct{vars: Rng /* NamedTypeVis */, bases: Option<Rng> /* TypeId */},
  Tuple {vars: Rng /* TypeId */},

  Iface{funs: Rng /* NamedTypeVis */, bases: Option<Rng> /* TypeId */},
  Trait{funs: Rng /* NamedTypeVis */, bases: Option<Rng> /* TypeId */},

  Fun {args: Rng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
  Init{args: Rng /* NamedType */, attr: u8 /* FunAttrs */},
  Fini{args: Rng /* NamedType */, attr: u8 /* FunAttrs */},

  Enum {vals: Rng /* Name | NamedExpr */, bases: Option<Rng> /* TypeId */},
  Flags{vals: Rng /* Name | NamedExpr */, bases: Option<Rng> /* TypeId */},

  Specialize{base: TypeId, args: Rng /* TypeId | ExprId */},
}
