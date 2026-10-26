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

use crate::{Ctx, builder::{ExprEmit, FunBuilder}, expr_p::ExprLow, type_p::TypeLow};


pub fn low_bool_condition(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, op: hir::BoolConditionOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<Value> {
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let lhs = ExprLow::low(ctx, sum, bbld, lhs)?.unwrap();
  let rhs = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();

  let op = match op {
    hir::BoolConditionOp::Eq   => mir::IntConditionOp::Eq,
    hir::BoolConditionOp::Ne   => mir::IntConditionOp::Ne,
  };


  // Post
  let this = mir::Expr::IntCondition {
    op,
    flg2: qwc_mir::IntConditionFlg2::Unsigned,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap();

  Success(this.into())
}

pub fn low_bool_logic(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, op: hir::BoolLogicOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Status<Value> {
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let lhs = ExprLow::low(ctx, sum, bbld, lhs)?.unwrap();
  let rhs = ExprLow::low(ctx, sum, bbld, rhs)?.unwrap();

  let op = match op {
    hir::BoolLogicOp::And => mir::IntLogicOp::And,
    hir::BoolLogicOp::Or  => mir::IntLogicOp::Or,
    hir::BoolLogicOp::Xor => mir::IntLogicOp::Xor,
  };


  // Post
  let this = mir::Expr::IntLogic {
    op,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap();

  Success(this.into())
}

pub fn low_bool_not(ctx: &mut Ctx, sum: &mut Summary, bbld: &mut FunBuilder, val: hir::ExprId) -> Status<Value> {
  let kind = TypeLow::low(ctx, sum, (ctx.src.get(val) as &hir::Expr).ety)?;
  let val = ExprLow::low(ctx, sum, bbld, val)?.unwrap();

  // Post
  let this = mir::Expr::IntUnary {
    op: qwc_mir::IntUnaryOp::Not,
    kind,
    val,
  }.emit(bbld).unwrap();

  Success(this.into())
}
