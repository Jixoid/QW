/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use itertools::izip;
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, ExprCategory};
use qwc_ast::{self as ast, Ident};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;

use super::helper;
use crate::{ExprLow, expr_p::const_p, hgen::Ctx};


// Unary
pub fn low_unary(ctx: &mut Ctx, op: ast::UnaryOp, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  use ast::UnaryOp::*;
  
  match op {
    Deref => low_unary_deref(ctx, id),

    Ref => low_unary_ref(ctx, id),

    Not => low_unary_not(ctx, id),
    
    _ => todo!("{op:#?}")
  }
}

fn low_unary_deref(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, id)?;
  let it = ctx.cre.get(id);
  let ty = ctx.cre.get(it.ety);

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

fn low_unary_ref(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, id)?;
  let it = ctx.cre.get(id);

  let ty = ctx.tin.ty_ref(ctx.cre, it.ety, false);

  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Ref(id),
    category: ExprCategory::RValue,
    ety: ty,
  };

  Ok(ctx.cre.push(this))
}

fn low_unary_not(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, id)?;
  let it = ctx.cre.get(id);
  let ty = ctx.cre.get(it.ety);

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
  let lhs_ex = ctx.cre.get(lhs);
  let rhs_ex = ctx.cre.get(rhs);

  let lhs_ty = ctx.cre.get(lhs_ex.ety);

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
  let lhs_ex = ctx.cre.get(lhs);
  let rhs_ex = ctx.cre.get(rhs);

  let lhs_ty = ctx.cre.get(lhs_ex.ety);

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
  let lhs_ex = ctx.cre.get(lhs);
  let rhs_ex = ctx.cre.get(rhs);

  let lhs_ty = ctx.cre.get(lhs_ex.ety);

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

  let lhs_ty = ctx.cre.get(lhs_hir).ety;
  let rhs_ty = ctx.cre.get(rhs_hir).ety;

  if lhs_ty != rhs_ty {
    todo!("{:#?} != {:#?}", ctx.cre.get(lhs_ty), ctx.cre.get(rhs_ty))
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

  let lhs_ty = ctx.cre.get(lhs_hir).ety;
  let rhs_ty = ctx.cre.get(rhs_hir).ety;

  if lhs_ty != rhs_ty {
    todo!("{:#?} != {:#?}", ctx.cre.get(lhs_ty), ctx.cre.get(rhs_ty))
  }


  // Op
  use ast::BinaryOp::*;
  
  let id = match op {
    // Arithmetic
    Add | Sub | Mul | Div | Rem => low_binary_arithmetic(ctx, op, lhs_hir, rhs_hir)?,

    _ => todo!("{op:#?}")
  };

  let ex = ctx.cre.get_mut(id);

  ex.kind = match ex.kind {
    hir::ExprKind::IntArithmetic{op, flg, lhs, rhs} => hir::ExprKind::AssignIntArithmetic{op, flg, lhs, rhs},
    
    _ => unreachable!()
  };

  Ok(id)
}


// Call & Index
pub fn low_call(ctx: &mut Ctx, it: &ast::Expr, callee: ast::ExprId, args: ast::ExprRng) -> Result<hir::ExprId, Message> {
  let callee = ExprLow::low(ctx, callee)?;
  
  let args_hir = {
    let mut vec = vec![];

    for id in ctx.src.extra_get(args) {
      let id = ExprLow::low(ctx, id)?;

      vec.push(id);
    }

    ctx.cre.extra(&vec)
  };
  
  let callee_ex = ctx.cre.get(callee);
  let callee_ty = ctx.cre.get(callee_ex.ety);

  let qwc_hir::TypeKind::Fun{self_kind: _, args: trait_args, ret: trait_ret} = callee_ty.kind else { panic!() };

  if args_hir.count() != trait_args.count() {
    let extra1 = if trait_args.count() > args_hir.count() {
      Some(X_ARGUMENTS_ARE_MISSING.args(&[&(trait_args.count()-args_hir.count()).to_string()]))
    } else {
      None
    };

    return Err(Message::error(FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED
      .args(&[
        &trait_args.count().to_string(),
        &args_hir.count().to_string(),
      ]),
      Label::new_pos(it.pos))
      .add_if(extra1)
    )
  }


  for (supp_hir, supp_ast, trt) in izip!(ctx.cre.extra_get(args_hir), ctx.src.extra_get(args), ctx.cre.extra_get(trait_args)) {
    let hir::Thing::NamedType(_, trt_ty) = *ctx.cre.get(trt) else { panic!() };
    
    let supp_pos = ctx.src.get(supp_ast).pos;
    let supp_ty = ctx.cre.get(supp_hir).ety;

    if supp_ty != trt_ty {
      return Err(Message::error(MISMATCHED_TYPES, Label::new_pos(supp_pos)))
    }
  }


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Call { callee, args: args_hir },
    category: ExprCategory::RValue,
    ety: trait_ret,
  };

  Ok(ctx.cre.push(this))
}


// Field Create
pub fn low_field_create(ctx: &mut Ctx, lhs: ast::ExprId, fields: ast::ThingRng, brace_span: Span) -> Result<hir::ExprId, Message> {
  let lhs = ExprLow::low(ctx, lhs)?;
  
  let hir::ExprKind::TypeOf{kind} = ctx.cre.get(lhs).kind else {
    return Err(Message::error(ONLY_TYPES_CAN_BE_INITIALIZED_IN_THIS_WAY, Label::new_pos(brace_span)))
  };


  // Get Keys
  let (keys, sort) = {
    let hir::TypeKind::Struct(fields) = ctx.cre.get(kind).kind else { panic!() };
    
    let mut keys = FxHashMap::default();
    let mut sort = vec![];
    
    for id in ctx.cre.extra_get(fields) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      
      keys.insert(name, kind);
      sort.push(name);
    }
    
    (keys, sort)
  };


  // Gived Keys
  let gived_keys = {
    let mut gived_keys = FxHashMap::default();

    for id in ctx.src.extra_get(fields) {
      let ast::Thing::NamedExpr(name, expr) = *ctx.src.get(id) else { panic!() };
      
      let expr_pos = ctx.src.get(expr).pos;
      let expr = ExprLow::low(ctx, expr)?;
      let expr_ty = ctx.cre.get(expr).ety;

      if let Some(&kind) = keys.get(&name.sid()) {
        if kind != expr_ty {
          return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(kind), &ctx.type_name(expr_ty)]), Label::new_pos(expr_pos)))
        }
      } else {
        return Err(Message::error(HAS_NO_FIELD_NAMED_X.args(&[name.str(ctx.far)]), Label::new_pos(name)))
      }


      use std::collections::hash_map::Entry::*;
      match gived_keys.entry(name.sid()) {
        Vacant(entry) => {entry.insert((<Ident as Into<Span>>::into(name), expr));},

        Occupied(entry) => {
          return Err(Message::error(FIELD_X_SPECIFIED_MORE_THAN_ONCE.args(&[name.str(ctx.far)]), Label::new_pos(name))
            .add(Label::new(entry.get().0, FIRST_DEFINITION_HERE))
          )
        }
      }
    }

    gived_keys
  };


  // Missing Keys
  {
    let missing_keys: Vec<Sid> = keys.iter().map(|(v, _)| v).filter(|&k| !gived_keys.contains_key(k)).copied().collect();

    if !missing_keys.is_empty() {
      let mut emsg = String::new();

      for sid in missing_keys {
        emsg += &format!("`{}` ", ctx.sin.str(sid));
      }
      emsg.pop();

      return Err(Message::error(MISSING_FIELDS_X_ININITIALIZER.args(&[&emsg]), Label::new_pos(brace_span)))
    }
  };


  // Resort
  let sorted: Vec<hir::ExprId> = sort.iter().map(|sid| gived_keys.get(sid).unwrap().1).collect();

  let fields = ctx.cre.extra(&sorted);


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::CombinatedInit { kind, fields },
    category: ExprCategory::RValue,
    ety: kind,
  };

  Ok(ctx.cre.push(this))
}


// Member
pub fn low_member(ctx: &mut Ctx, rng: ast::ExprRng) -> Result<hir::ExprId, Message> {
  let ids: Vec<ast::ExprId> = ctx.src.extra_get(rng).collect();
  let mut current_id = ExprLow::low(ctx, ids[0])?;

  for &field_expr_id in &ids[1..] {
    let field_expr = ctx.src.get(field_expr_id);
    let ast::ExprKind::Nick(ident) = field_expr.kind else { panic!("expected field identifier"); };

    let current = ctx.cre.get(current_id);

    let hir::TypeKind::Struct(fields) = ctx.cre.get(current.ety).kind else {
      return Err(Message::error(HAS_NO_FIELD_NAMED_X.args(&[ident.str(ctx.far)]), Label::new_pos(ident)));
    };

    let mut found = None;
    for (idx, field_thing_id) in ctx.cre.extra_get(fields).enumerate() {
      let hir::Thing::NamedType(name, fty) = *ctx.cre.get(field_thing_id) else { panic!() };
      if name == ident.sid() {
        found = Some((idx as u32, fty));
        break;
      }
    }

    let (field_idx, field_ty) = match found {
      Some(f) => f,
      None => return Err(Message::error(HAS_NO_FIELD_NAMED_X.args(&[ident.str(ctx.far)]), Label::new_pos(ident))),
    };

    let this = hir::Expr {
      kind: hir::ExprKind::Field {
        target: current_id,
        idx: field_idx,
      },
      category: current.category,
      ety: field_ty,
    };

    current_id = ctx.cre.push(this);
  }

  Ok(current_id)
}
