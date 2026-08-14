use crate::{ast::{ExprId, Rng, TypeId, Visibility}, lexer::Span};


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }


#[derive(Clone)]
pub struct FieldType {
  pub name: Span,
  pub kind: TypeId,
  pub vis: Visibility,
  pub attrs: Vec<crate::ast::Attr>,
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IntegerValue { SIG(i64), USG(u64) }


#[derive(Clone)]
pub struct NickType {
  pub pos: Span,
  pub idx: u32
}


pub enum Type {
  Nick(NickType),
  Path(Rng),

  Ptr   {sub: TypeId, acc: AccessKind},
  Ref   {sub: TypeId, acc: AccessKind},
  Array {sub: TypeId, ext: Option<Rng> /* ExprId */},
  Vector{sub: TypeId, ext: ExprId},
  Range {sub: TypeId},
  Option{sub: TypeId},
  Result{sub: TypeId, err: TypeId},
  
  Struct{vars: Vec<FieldType>},
  Tuple {vars: Rng /* TypeId */},

  Iface{funs: Vec<FieldType>},
  Trait{funs: Vec<FieldType>},

  Fun {args: Vec<FieldType>, ret: Option<TypeId>},
  Init{args: Vec<FieldType>, ils: Option<Rng> /* NamedTypeList */},

  Enum {vals: Rng /* Name | NamedExpr */},
  Flags{vals: Rng /* Name | NamedExpr */},

  Specialize{base: TypeId, args: Vec<TypeId>},
}
