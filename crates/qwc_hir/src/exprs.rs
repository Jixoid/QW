/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_string_interner::Sid;
use rustc_apfloat::ieee;

use crate::{ExprId, ExprRng, ItemId, TypeId};


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Const {
  // ZST
  Unit,

  // Primitive
  Bool(bool),
  Str(Sid),
  Char(char),
  
  // Integer
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  U128(u128),
  
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
  I128(i128),

  // Float
  F16(ieee::Half),
  F32(ieee::Single),
  F64(ieee::Double),
  F128(ieee::Quad),

  // Array
  Array(TypeId, ExprRng /* must be const */),
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



#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExprKind {
  GenericExpr{generic: ItemId, idx: usize},
  GenericRaw{generic: ItemId, kind: ExprId},
  Error,
  
  // Const
  Const(Const),

  // Addr
  Ref(ExprId, bool /* ism */),
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
  Return(ExprId),
  Break(Option<ExprId>),
  Continue,

  // Branch
  If{cond: ExprId, then: ExprId, elsb: Option<ExprId>},

  // Call
  Call{callee: ExprId, args: ExprRng},
  
  BoundSelfMethod{callee: ExprId, this: ExprId},
  
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

  /// Primitive integer cast (x as i64, etc.)
  IntCast(ExprId),

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


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Expr {
  pub kind: ExprKind,
  
  pub category: ExprCategory,

  /// Evaluated Type
  pub ety: TypeId
}
