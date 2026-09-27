use std::assert_matches;

use qwc_diagnostic::{Message, Span};
use qwc_hir::{self as hir, ExprCategory};
use qwc_ast as ast;

use super::helper;
use crate::{ExprLow, expr_p::const_p, hgen::Ctx};


// Unary
pub fn low_unary(ctx: &mut Ctx, op: ast::UnaryOp, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  use ast::UnaryOp::*;
  
  match op {
    Deref => low_unary_deref(ctx, id),

    Not => low_unary_not(ctx, id),
    
    _ => todo!("{op:#?}")
  }
}

fn low_unary_deref(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, id)?;
  let it: &hir::Expr = ctx.cre.get(id);
  let ty: &hir::Type = ctx.cre.get(it.ety);

  use hir::TypeKind::*;

  let (ty, ism) = match ty.kind {
    Ref(ty, ism) => (ty, ism),

    _ => todo!("{ty:#?}")
  };


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Deref(id),
    category: ExprCategory::lvalue(ism),
    ety: ty,
  };

  Ok(ctx.cre.push(this))
}

fn low_unary_not(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, id)?;
  let it: &hir::Expr = ctx.cre.get(id);
  let ty: &hir::Type = ctx.cre.get(it.ety);

  use hir::TypeKind::*;

  assert_matches!(ty.kind, Bool{..});


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::BoolNot(id),
    category: ExprCategory::RValue,
    ety: it.ety,
  };

  Ok(ctx.cre.push(this))
}


// Binary
pub fn low_binary(ctx: &mut Ctx, op: ast::BinaryOp, lhs: ast::ExprId, rhs: ast::ExprId) -> Result<hir::ExprId, Message> {
  let lhs = ExprLow::low(ctx, lhs)?;
  let rhs = ExprLow::low(ctx, rhs)?;
  
  use ast::BinaryOp::*;
  
  let ex = match op {
    // Arithmetic
    Add | Sub | Mul | Div | Rem => low_binary_arithmetic(ctx, op, lhs, rhs)?,

    Gt | Lt | GtEq | LtEq | Eq | Ne => low_binary_condition(ctx, op, lhs, rhs)?,

    And | Or | Xor => low_binary_logic(ctx, op, lhs, rhs)?,

    _ => todo!("{op:#?}")
  };

  Ok(ex)
}

fn low_binary_arithmetic(ctx: &mut Ctx, op: ast::BinaryOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<hir::ExprId, Message> {
  let lhs_ex: &hir::Expr = ctx.cre.get(lhs);
  let rhs_ex: &hir::Expr = ctx.cre.get(rhs);

  let lhs_ty: &hir::Type = ctx.cre.get(lhs_ex.ety);

  if lhs_ex.ety != rhs_ex.ety { panic!() }

  if lhs_ex.ety != rhs_ex.ety { panic!() }

  if !matches!(lhs_ty.kind, hir::TypeKind::ArchInt(..) | qwc_hir::TypeKind::Int(..) ) { panic!() }

  use ast::BinaryOp::*;
  
  let op = match op {
    Add => hir::IntArithmeticOp::Add,
    Sub => hir::IntArithmeticOp::Sub,
    Mul => hir::IntArithmeticOp::Mul,
    Div => hir::IntArithmeticOp::Div,
    Rem => hir::IntArithmeticOp::Rem,
    _ => panic!()
  };
  
  let flg = hir::IntArithmeticFlg::Overflow;


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::IntArithmetic{ op, flg, lhs, rhs },
    category: ExprCategory::RValue,
    ety: lhs_ex.ety,
  };
  
  Ok(ctx.cre.push(this))
}

fn low_binary_condition(ctx: &mut Ctx, op: ast::BinaryOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<hir::ExprId, Message> {
  let lhs_ex: &hir::Expr = ctx.cre.get(lhs);
  let rhs_ex: &hir::Expr = ctx.cre.get(rhs);

  let lhs_ty: &hir::Type = ctx.cre.get(lhs_ex.ety);

  if lhs_ex.ety != rhs_ex.ety { panic!() }

  if !matches!(lhs_ty.kind, hir::TypeKind::ArchInt(..) | qwc_hir::TypeKind::Int(..) ) { panic!() }

  use ast::BinaryOp::*;
  
  let op = match op {
    Gt   => hir::IntConditionOp::Gt,
    Lt   => hir::IntConditionOp::Lt,
    GtEq => hir::IntConditionOp::GtEq,
    LtEq => hir::IntConditionOp::LtEq,
    Eq   => hir::IntConditionOp::Eq,
    Ne   => hir::IntConditionOp::Ne,
    _ => panic!()
  };


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::IntCondition{ op, lhs, rhs },
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_bool(),
  };
  
  Ok(ctx.cre.push(this))
}

fn low_binary_logic(ctx: &mut Ctx, op: ast::BinaryOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<hir::ExprId, Message> {
  let lhs_ex: &hir::Expr = ctx.cre.get(lhs);
  let rhs_ex: &hir::Expr = ctx.cre.get(rhs);

  let lhs_ty: &hir::Type = ctx.cre.get(lhs_ex.ety);

  if lhs_ex.ety != rhs_ex.ety { panic!() }

  if !matches!(lhs_ty.kind, hir::TypeKind::Bool{..} ) { panic!() }

  use ast::BinaryOp::*;
  
  let kind = match op {
    And => hir::ExprKind::If {
      cond: lhs,
      then: rhs,
      elsb: Some(const_p::low_bool(ctx, false)?),
    },

    Or => hir::ExprKind::If {
      cond: lhs,
      then: const_p::low_bool(ctx, true)?,
      elsb: Some(rhs),
    },

    Xor => hir::ExprKind::BoolLogic {
      op: hir::BoolLogicOp::Xor,
      lhs,
      rhs,
    },

    _ => unreachable!()
  };


  // Post
  let this = hir::Expr {
    kind,
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_bool(),
  };

  Ok(ctx.cre.push(this))
}



// Assign
pub fn low_assign(ctx: &mut Ctx, lhs: ast::ExprId, rhs: ast::ExprId, op_span: Span) -> Result<hir::ExprId, Message> {
  let lhs_hir = ExprLow::low(ctx, lhs)?;
  let rhs_hir = ExprLow::low(ctx, rhs)?;

  helper::verify_assignable(ctx, lhs, lhs_hir, op_span)?;

  let lhs_ty = (ctx.cre.get(lhs_hir) as &hir::Expr).ety;
  let rhs_ty = (ctx.cre.get(rhs_hir) as &hir::Expr).ety;

  if lhs_ty != rhs_ty {
    todo!("{:#?} != {:#?}", ctx.cre.get(lhs_ty) as &hir::Type, ctx.cre.get(rhs_ty) as &hir::Type)
  }

  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Assign {lhs: lhs_hir, rhs: rhs_hir},
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_unit(),
  };
  
  Ok(ctx.cre.push(this))
}


// Assign Op
pub fn low_assign_op(ctx: &mut Ctx, op: ast::BinaryOp, lhs: ast::ExprId, rhs: ast::ExprId, op_span: Span) -> Result<hir::ExprId, Message> {
  let lhs_hir = ExprLow::low(ctx, lhs)?;
  let rhs_hir = ExprLow::low(ctx, rhs)?;

  helper::verify_assignable(ctx, lhs, lhs_hir, op_span)?;

  let lhs_ty = (ctx.cre.get(lhs_hir) as &hir::Expr).ety;
  let rhs_ty = (ctx.cre.get(rhs_hir) as &hir::Expr).ety;

  if lhs_ty != rhs_ty {
    todo!("{:#?} != {:#?}", ctx.cre.get(lhs_ty) as &hir::Type, ctx.cre.get(rhs_ty) as &hir::Type)
  }


  // Op
  use ast::BinaryOp::*;
  
  let id = match op {
    // Arithmetic
    Add | Sub | Mul | Div | Rem => low_binary_arithmetic(ctx, op, lhs_hir, rhs_hir)?,

    _ => todo!("{op:#?}")
  };

  let ex: &mut hir::Expr = ctx.cre.get_mut(id);

  ex.kind = match ex.kind {
    hir::ExprKind::IntArithmetic{op, flg, lhs, rhs} => hir::ExprKind::AssignIntArithmetic{op, flg, lhs, rhs},
    
    _ => unreachable!()
  };

  Ok(id)
}
