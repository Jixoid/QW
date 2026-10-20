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
