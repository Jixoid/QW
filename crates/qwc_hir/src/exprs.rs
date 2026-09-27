use crate::{ExprId, ExprRng, ItemId, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Const {
  // ZST
  Unit,

  // Primitive
  Bool(bool),
  Int(i32),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntArithmeticFlg {
  Overflow, Checked, Saturating
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntArithmeticOp {
  Add, Sub, Mul, Div, Rem
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntConditionOp {
  Gt, Lt, GtEq, LtEq, Eq, Ne
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BoolLogicOp {
  And, Or, Xor
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
  Block{stmt: ExprRng, expr: Option<ExprId>},

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

  // Integer
  IntArithmetic{op: IntArithmeticOp, flg: IntArithmeticFlg, lhs: ExprId, rhs: ExprId},
  AssignIntArithmetic{op: IntArithmeticOp, flg: IntArithmeticFlg, lhs: ExprId, rhs: ExprId},

  IntCondition{op: IntConditionOp, lhs: ExprId, rhs: ExprId},

  // Bool
  BoolLogic{op: BoolLogicOp, lhs: ExprId, rhs: ExprId},
  BoolNot(ExprId),
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
