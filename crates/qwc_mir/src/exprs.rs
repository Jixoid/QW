/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{SymbId, TypeId, ValueRng};



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
  Param(u32),
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
  Alloca {kind: TypeId},
  
  Store {target: Value, kind: TypeId, value: Value},
  Load {target: Value, kind: TypeId},

  Call {callee: Value, args: ValueRng},

  Gep {target: Value, kind: TypeId, idx: u32},

  IntArithmetic {op: IntArithmeticOp, flg: IntArithmeticFlg, flg2: IntArithmeticFlg2, kind: TypeId, lhs: Value, rhs: Value},
  IntCondition {op: IntConditionOp, flg2: IntConditionFlg2, kind: TypeId, lhs: Value, rhs: Value},
  IntLogic {op: IntLogicOp, kind: TypeId, lhs: Value, rhs: Value},
  IntUnary {op: IntUnaryOp, kind: TypeId, val: Value},
}

impl Expr {
  pub fn have_result(&self) -> bool {
    match self {
      Expr::Alloca{..} => true,

      Expr::Load{..} => true,
      Expr::Store{..} => false,

      Expr::Call{..} => true,

      Expr::Gep{..} => true,

      Expr::IntArithmetic{..} => true,
      Expr::IntCondition{..} => true,
      Expr::IntLogic{..} => true,
      Expr::IntUnary{..} => true,
    }
  }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
  pub kind: Expr,
  pub dest: Option<SSA>,
}
