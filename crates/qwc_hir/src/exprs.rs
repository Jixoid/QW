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

  Deref(ExprId),

  // Ref
  GlobalRef(ItemId),
  LocalRef(u32),
  
  // Variable
  Let{local: u32, init: ExprId},

  // Code
  Block{stmt: Rng /* ExprId */, expr: Option<ExprId>},

  // Assign
  Assign{lhs: ExprId, rhs: ExprId},

  // Loop
  Loop{blok: ExprId, elsb: Option<ExprId>},

  // Route
  Return(Option<ExprId>),
  Break(Option<ExprId>),
  Continue,

  // Branch
  If{cond: ExprId, then: ExprId, elsb: Option<ExprId>},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprCategory {
  RValue,
  LValueMut,
  LValueImm,
}

impl ExprCategory {
  pub fn lvalue(ism: bool) -> Self {
    if ism { ExprCategory::LValueMut } else { ExprCategory::LValueImm }
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
  pub kind: ExprKind,
  
  pub category: ExprCategory,

  /// Evaluated Type
  pub ety: TypeId
}
