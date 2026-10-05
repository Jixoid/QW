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
  Eq, Ne, Gt, Lt, GtEq, LtEq
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FloatArithmeticOp {
  Add, Sub, Mul, Div, Rem
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BoolLogicOp {
  And, Or, Xor
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
  GenericExpr,
  
  Const(Const),

  Ref(ExprId),
  Deref(ExprId),

  // Ref
  GlobalRef(ItemId),
  LocalRef(u32),
  
  // Variable
  Let{local: u32, init: ExprId},

  // Code
  Block{stmt: ExprRng, expr: Option<ExprId>},

  // TypeOf
  TypeOf{kind: TypeId},

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

  // Call
  Call{callee: ExprId, args: ExprRng},

  // Field
  Field{target: ExprId, idx: u32},
  CombinatedInit{kind: TypeId, fields: ExprRng},

  // Integer
  IntArithmetic{op: IntArithmeticOp, flg: IntArithmeticFlg, lhs: ExprId, rhs: ExprId},
  AssignIntArithmetic{op: IntArithmeticOp, flg: IntArithmeticFlg, lhs: ExprId, rhs: ExprId},

  IntCondition{op: IntConditionOp, lhs: ExprId, rhs: ExprId},

  // Floating
  FloatArithmetic{op: FloatArithmeticOp, lhs: ExprId, rhs: ExprId},

  // Bool
  BoolLogic{op: BoolLogicOp, lhs: ExprId, rhs: ExprId},
  BoolNot(ExprId),

  // Cast

  /// &Type => &Iface
  CastToIfaceRef{ref_of_expr: ExprId, ref_of_type: TypeId, target_iface: TypeId},
  
  /// Type => Trait
  CastToTrait{expr: ExprId, target_trait: TypeId},
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

  pub fn is_lvalue(&self) -> bool {
    matches!(self, ExprCategory::LValueImm | ExprCategory::LValueMut)
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
  pub kind: ExprKind,
  
  pub category: ExprCategory,

  /// Evaluated Type
  pub ety: TypeId
}
