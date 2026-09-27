use crate::{SymbId, TypeId};



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SSA (
  pub(crate) u32
);

impl SSA {
  pub fn new(val: u32) -> Self {
    Self(val)
  }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Value {
  SSA(SSA),
  Const(Const),
  GlobalRef(SymbId),
  StackRef(u32),
}

impl Into<Value> for SSA {
  fn into(self) -> Value { Value::SSA(self) }
}

impl Into<Value> for Const {
  fn into(self) -> Value { Value::Const(self) }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Const {
  Unit,
  
  Bool(bool),
  Int(i32),
}




#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntArithmeticOp { Add, Sub, Mul, Div, Rem }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntArithmeticFlg { Overflow, Checked, Saturating }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntArithmeticFlg2 { Signed, Unsigned }



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntConditionOp { GtEq, LtEq, Gt, Lt, Eq, Ne }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntConditionFlg2 { Signed, Unsigned }



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntLogicOp { And, Or, Xor }



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntUnaryOp { Not }



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
  Store{target: Value, kind: TypeId, value: Value},
  Load{target: Value, kind: TypeId},

  IntArithmetic{op: IntArithmeticOp, flg: IntArithmeticFlg, flg2: IntArithmeticFlg2, kind: TypeId, lhs: Value, rhs: Value},
  IntCondition{op: IntConditionOp, flg2: IntConditionFlg2, kind: TypeId, lhs: Value, rhs: Value},
  IntLogic{op: IntLogicOp, kind: TypeId, lhs: Value, rhs: Value},
  IntUnary{op: IntUnaryOp, kind: TypeId, val: Value},
}

impl Expr {
  pub fn have_result(&self) -> bool {
    match self {
      Expr::Load{..} => true,
      Expr::IntArithmetic{..} => true,
      Expr::IntCondition{..} => true,
      Expr::IntLogic{..} => true,
      Expr::IntUnary{..} => true,

      Expr::Store{..} => false,
    }
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
  pub kind: Expr,
  pub dest: Option<SSA>,
}
