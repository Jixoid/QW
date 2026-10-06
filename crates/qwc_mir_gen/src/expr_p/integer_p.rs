use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Value};

use crate::{Ctx, ExprLow, FunBuilder, TypeLow, ExprEmit};


pub fn low_int_arithmetic(ctx: &mut Ctx, bbld: &mut FunBuilder, op: hir::IntArithmeticOp, flg: hir::IntArithmeticFlg, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<Value, Message> {
  let kind = TypeLow::low(ctx, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let lhs = ExprLow::low(ctx, bbld, lhs)?.unwrap();
  let rhs = ExprLow::low(ctx, bbld, rhs)?.unwrap();
  let kind_ty = ctx.cre.get(kind);


  let op = match op {
    hir::IntArithmeticOp::Add => mir::IntArithmeticOp::Add,
    hir::IntArithmeticOp::Sub => mir::IntArithmeticOp::Sub,
    hir::IntArithmeticOp::Mul => mir::IntArithmeticOp::Mul,
    hir::IntArithmeticOp::Div => mir::IntArithmeticOp::Div,
    hir::IntArithmeticOp::Rem => mir::IntArithmeticOp::Rem,
  };

  assert_eq!(flg, hir::IntArithmeticFlg::Overflow);
  let flg = mir::IntArithmeticFlg::Overflow;

  let flg2 = match kind_ty.kind {
    mir::TypeKind::Int(_, true)  => mir::IntArithmeticFlg2::Signed,
    mir::TypeKind::Int(_, false) => mir::IntArithmeticFlg2::Unsigned,
    
    _ => unreachable!()
  };


  // Post
  let this = mir::Expr::IntArithmetic {
    op,
    flg,
    flg2,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap();

  Ok(this.into())
}

pub fn low_int_arithmetic_op(ctx: &mut Ctx, bbld: &mut FunBuilder, op: hir::IntArithmeticOp, flg: hir::IntArithmeticFlg, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<(), Message> {
  let target = ExprLow::low_lval(ctx, bbld, lhs)?;
  let kind = TypeLow::low(ctx, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let rhs = ExprLow::low(ctx, bbld, rhs)?.unwrap();
  let kind_ty: &mir::Type = ctx.cre.get(kind);
  
  let op = match op {
    hir::IntArithmeticOp::Add => mir::IntArithmeticOp::Add,
    hir::IntArithmeticOp::Sub => mir::IntArithmeticOp::Sub,
    hir::IntArithmeticOp::Mul => mir::IntArithmeticOp::Mul,
    hir::IntArithmeticOp::Div => mir::IntArithmeticOp::Div,
    hir::IntArithmeticOp::Rem => mir::IntArithmeticOp::Rem,
  };

  assert_eq!(flg, hir::IntArithmeticFlg::Overflow);
  let flg = mir::IntArithmeticFlg::Overflow;

  let flg2 = match kind_ty.kind {
    mir::TypeKind::Int(_, true)  => mir::IntArithmeticFlg2::Signed,
    mir::TypeKind::Int(_, false) => mir::IntArithmeticFlg2::Unsigned,
    
    _ => unreachable!()
  };


  // Post
  let lhs = mir::Expr::Load {
    target,
    kind,
  }.emit(bbld).unwrap().into();

  let value = mir::Expr::IntArithmetic {
    op,
    flg,
    flg2,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap().into();

  mir::Expr::Store {
    target,
    kind,
    value,
  }.emit(bbld);

  Ok(())
}


pub fn low_int_condition(ctx: &mut Ctx, bbld: &mut FunBuilder, op: hir::IntConditionOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<Value, Message> {
  let kind = TypeLow::low(ctx, (ctx.src.get(lhs) as &hir::Expr).ety)?;
  let lhs = ExprLow::low(ctx, bbld, lhs)?.unwrap();
  let rhs = ExprLow::low(ctx, bbld, rhs)?.unwrap();
  let kind_ty: &mir::Type = ctx.cre.get(kind);

  let op = match op {
    hir::IntConditionOp::GtEq => mir::IntConditionOp::GtEq,
    hir::IntConditionOp::LtEq => mir::IntConditionOp::LtEq,
    hir::IntConditionOp::Gt   => mir::IntConditionOp::Gt,
    hir::IntConditionOp::Lt   => mir::IntConditionOp::Lt,
    hir::IntConditionOp::Eq   => mir::IntConditionOp::Eq,
    hir::IntConditionOp::Ne   => mir::IntConditionOp::Ne,
  };

  let flg2 = match kind_ty.kind {
    mir::TypeKind::Int(_, true)  => mir::IntConditionFlg2::Signed,
    mir::TypeKind::Int(_, false) => mir::IntConditionFlg2::Unsigned,
    
    _ => unreachable!()
  };


  // Post
  let this = mir::Expr::IntCondition {
    op,
    flg2,
    kind,
    lhs,
    rhs
  }.emit(bbld).unwrap();

  Ok(this.into())
}
