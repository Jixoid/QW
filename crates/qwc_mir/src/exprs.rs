/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_string_interner::Sid;
use rustc_apfloat::ieee;

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



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
  SSA(SSA),
  Const(Const),
  GlobalRef(SymbId),
  Param(u32),
}

impl From<SSA> for Value {
  fn from(val: SSA) -> Self { Value::SSA(val) }
}

impl From<Const> for Value {
  fn from(val: Const) -> Self { Value::Const(val) }
}



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Const {
  Unit,
  
  Bool(bool),
  
  I8(u8),
  I16(u16),
  I32(u32),
  I64(u64),
  I128(u128),
  
  F16(ieee::Half),
  F32(ieee::Single),
  F64(ieee::Double),
  F128(ieee::Quad),

  Array{kind: TypeId, rng: ValueRng},
  
  Str(Sid),
}


mod int_s {
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
  pub enum IntUnaryOp { Not, Neg }
}
pub use int_s::*;


mod float_s {
  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub enum FloatArithmeticOp { Add, Sub, Mul, Div, Rem }

  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub enum FloatUnaryOp { Neg }
}
pub use float_s::*;



#[derive(Debug, Copy, Clone, PartialEq)]
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
  IntCast {kind: TypeId, val: Value, signed: bool},

  FloatArithmetic {op: FloatArithmeticOp, kind: TypeId, lhs: Value, rhs: Value},
  FloatUnary {op: FloatUnaryOp, kind: TypeId, val: Value},
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
      Expr::IntCast{..} => true,

      Expr::FloatArithmetic{..} => true,
      Expr::FloatUnary{..} => true,
    }
  }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Inst {
  pub kind: Expr,
  pub dest: Option<SSA>,
}
