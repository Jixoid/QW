use std::num::NonZeroU32;

use crate::hir::{AccessKind, ExprId, Rng, TypeId};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
  Add, Sub, Mul, Div, Rem,
  
  Eq, Ne, Lt, Gt, Lte, Gte,
  
  And, Or, Xor, Shl, Shr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
  Neg, Not, BitNot, Ref(AccessKind), Ptr(AccessKind), Deref,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
  Int(i64),
  Float(u64),
  Bool(bool),
  Char(char),
  Str(NonZeroU32 /* string pool sid */),
  Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprVari {
  Lit(Value),
  Value(TypeId),

  Unary  { op: UnaryOp, val: ExprId },
  Binary { op: BinaryOp, lhs: ExprId, rhs: ExprId },

  Assign  { lhs: ExprId, rhs: ExprId },
  AssignOp{ op: BinaryOp, lhs: ExprId, rhs: ExprId },
  
  Exchange{ lhs: ExprId, rhs: ExprId },

  Block { stmts: Rng /* ExprId */, expr: Option<ExprId> },
  If    { cond: ExprId, then_b: ExprId, else_b: Option<ExprId> },
  While { cond: ExprId, body: ExprId },
  Loop  { body: ExprId },

  Call  { callee: ExprId, args: Rng /* ExprId */ },
  Member{ base: ExprId, index: u32 },
  Index { base: ExprId, index: ExprId },

  Let   { local: u32, ty: TypeId, init: Option<ExprId>, acck: AccessKind },
  Ret   { val: Option<ExprId> },
  Break { val: Option<ExprId> },
  Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
  pub ty: TypeId,
  pub vari: ExprVari,
}
