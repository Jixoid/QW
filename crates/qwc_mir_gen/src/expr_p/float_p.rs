/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use qwc_diagnostic::{Status::{self, Success}, Summary};
use qwc_mir::{self as mir, Value};
use qwc_hir as hir;

use crate::{Ctx, ExprLow, FunBuilder, TypeLow, ExprEmit};


pub fn low_float_arithmetic(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, op: hir::FloatArithmeticOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<Value> {
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let lhs = ExprLow::low(ctx, sum, bbld, lhs)?.unwrap();
  let rhs = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();

  let op = match op {
    hir::FloatArithmeticOp::Add => mir::FloatArithmeticOp::Add,
    hir::FloatArithmeticOp::Sub => mir::FloatArithmeticOp::Sub,
    hir::FloatArithmeticOp::Mul => mir::FloatArithmeticOp::Mul,
    hir::FloatArithmeticOp::Div => mir::FloatArithmeticOp::Div,
    hir::FloatArithmeticOp::Rem => mir::FloatArithmeticOp::Rem,
  };


  // Post
  let this = mir::Expr::FloatArithmetic {
    op,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap();

  Success(this.into())
}

pub fn low_float_arithmetic_op(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, op: hir::FloatArithmeticOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<()> {
  let target = ExprLow::low_lval(ctx, sum, bbld, lhs)?;
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let rhs = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();
  
  let op = match op {
    hir::FloatArithmeticOp::Add => mir::FloatArithmeticOp::Add,
    hir::FloatArithmeticOp::Sub => mir::FloatArithmeticOp::Sub,
    hir::FloatArithmeticOp::Mul => mir::FloatArithmeticOp::Mul,
    hir::FloatArithmeticOp::Div => mir::FloatArithmeticOp::Div,
    hir::FloatArithmeticOp::Rem => mir::FloatArithmeticOp::Rem,
  };


  // Post
  let lhs = mir::Expr::Load {
    target,
    kind,
  }.emit(bbld).unwrap().into();

  let value = mir::Expr::FloatArithmetic {
    op,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap().into();

  mir::Expr::Store {
    target,
    kind,
    value,
  }.emit(bbld);

  Success(())
}


pub fn low_float_condition(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, op: hir::FloatConditionOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<Value> {
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let lhs = ExprLow::low(ctx, sum, bbld, lhs)?.unwrap();
  let rhs = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();
  //let kind_ty: &mir::Type = ctx.cre.get(kind);

  let (op, flg) = match op {
    hir::FloatConditionOp::GtEq => (mir::FloatConditionOp::GtEq, mir::FloatConditionFlg::Ordered),
    hir::FloatConditionOp::LtEq => (mir::FloatConditionOp::LtEq, mir::FloatConditionFlg::Ordered),
    hir::FloatConditionOp::Gt   => (mir::FloatConditionOp::Gt,   mir::FloatConditionFlg::Ordered),
    hir::FloatConditionOp::Lt   => (mir::FloatConditionOp::Lt,   mir::FloatConditionFlg::Ordered),
    hir::FloatConditionOp::Eq   => (mir::FloatConditionOp::Eq,   mir::FloatConditionFlg::Ordered),
    hir::FloatConditionOp::Ne   => (mir::FloatConditionOp::Ne,   mir::FloatConditionFlg::Unordered),
  };


  // Post
  let this = mir::Expr::FloatCondition {
    op,
    flg,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap();

  Success(this.into())
}


pub fn low_float_neg(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, val: hir::ExprId) -> Status<Value> {
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(val) as &hir::Expr).ety)?;
  let val = ExprLow::low(ctx, sum, bbld, val)?.unwrap();

  let this = mir::Expr::FloatUnary {
    op: mir::FloatUnaryOp::Neg,
    kind,
    val,
  }.emit(bbld).unwrap();

  Success(this.into())
}
