use crate::{ExprId, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Value {
  Bool(bool),
  Int(i32),
  
  // NST
  Unit,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
  GenericExpr,
  
  Lit(Value),

  Op(ExprId, ExprId),

  // Code
  Block{stack: Rng /* TypeId */, expr: Option<ExprId> },
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
  pub kind: ExprKind,
  
  /// Evaluated Type
  pub ety: TypeId
}
