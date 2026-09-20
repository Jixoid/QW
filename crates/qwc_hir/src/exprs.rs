use crate::{ExprId, ItemId, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Const {
  // ZST
  Unit,

  // Primitive
  Bool(bool),
  Int(i32),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
  GenericExpr,
  
  Const(Const),

  Op(ExprId, ExprId),

  // Memory
  GlobalRef(ItemId),
  Load(ExprId),
  Store{ptr: ExprId, val: ExprId},

  // Code
  Block{stmt: Rng /* ExprId */, expr: Option<ExprId>},

  // Assign
  Assign{lhs: ExprId, rhs: ExprId},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
  pub kind: ExprKind,
  
  /// Evaluated Type
  pub ety: TypeId
}
