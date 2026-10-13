/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_ast::{self as ast, Ident};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;

use super::helper;
use crate::{Ctx, ExprLow, TypeMatch, expr_p::const_p, FunCtx, Infer};


// Unary
pub fn low_unary(ctx: &mut Ctx, fctx: &FunCtx, op: ast::UnaryOp, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  use ast::UnaryOp::*;
  
  match op {
    Deref => low_unary_deref(ctx, fctx, id),
    
    Ref{ism} => low_unary_ref(ctx, fctx, id, ism),

    Not => low_unary_not(ctx, fctx, id),
    
    _ => todo!("{op:#?}")
  }
}

fn low_unary_deref(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, fctx, id)?;
  err_bypass!(ctx, id);
  let it = ctx.cre.get(id);
  let ty = ctx.get(it.ety);

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

fn low_unary_ref(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId, ism: bool) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, fctx, id)?;
  err_bypass!(ctx, id);
  let it = ctx.cre.get(id);

  let ty = ctx.tin.ty_ref(ctx.cre, ctx.ideps, it.ety, ism);

  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Ref(id, ism),
    category: ExprCategory::RValue,
    ety: ty,
  };

  Ok(ctx.cre.push(this))
}

fn low_unary_not(ctx: &mut Ctx, fctx: &FunCtx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
  let id = ExprLow::low(ctx, fctx, id)?;
  err_bypass!(ctx, id);
  let it = ctx.cre.get(id);
  let ty = ctx.get(it.ety);

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
pub fn low_binary(ctx: &mut Ctx, fctx: &FunCtx, pos: Span, op: ast::BinaryOp, lhs: ast::ExprId, rhs: ast::ExprId) -> Result<hir::ExprId, Message> {
  let lhs = ExprLow::low(ctx, fctx, lhs)?;
  let rhs = ExprLow::low(ctx, fctx, rhs)?;
  
  err_bypass!(ctx, lhs, rhs);

  use ast::BinaryOp::*;
  
  let ex = match op {
    // Arithmetic
    Add | Sub | Mul | Div | Rem => low_binary_arithmetic(ctx, pos, op, lhs, rhs)?,

    Gt | Lt | GtEq | LtEq | Eq | Ne => low_binary_condition(ctx, op, lhs, rhs)?,

    And | Or | Xor => low_binary_logic(ctx, op, lhs, rhs)?,

    _ => todo!("{op:#?}")
  };

  Ok(ex)
}

fn low_binary_arithmetic(ctx: &mut Ctx, pos: Span, op: ast::BinaryOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<hir::ExprId, Message> {
  let lhs_ex = ctx.cre.get(lhs);
  let rhs_ex = ctx.cre.get(rhs);

  let lhs_ty = ctx.get(lhs_ex.ety);

  if lhs_ex.ety == rhs_ex.ety {
    if matches!(lhs_ty.kind, hir::TypeKind::ArchInt(..) | hir::TypeKind::Int(..)) {
      use ast::BinaryOp::*;
      let aop = match op {
        Add => hir::IntArithmeticOp::Add,
        Sub => hir::IntArithmeticOp::Sub,
        Mul => hir::IntArithmeticOp::Mul,
        Div => hir::IntArithmeticOp::Div,
        Rem => hir::IntArithmeticOp::Rem,
        _ => unreachable!(),
      };
      let flg = hir::IntArithmeticFlg::Overflow;
      let this = hir::Expr {
        kind: hir::ExprKind::IntArithmetic { op: aop, flg, lhs, rhs },
        category: ExprCategory::RValue,
        ety: lhs_ex.ety,
      };
      return Ok(ctx.cre.push(this));
    }

    if matches!(lhs_ty.kind, hir::TypeKind::Float(..)) {
      use ast::BinaryOp::*;
      let aop = match op {
        Add => hir::FloatArithmeticOp::Add,
        Sub => hir::FloatArithmeticOp::Sub,
        Mul => hir::FloatArithmeticOp::Mul,
        Div => hir::FloatArithmeticOp::Div,
        Rem => hir::FloatArithmeticOp::Rem,
        _ => unreachable!(),
      };
      let this = hir::Expr {
        kind: hir::ExprKind::FloatArithmetic { op: aop, lhs, rhs },
        category: ExprCategory::RValue,
        ety: lhs_ex.ety,
      };
      return Ok(ctx.cre.push(this));
    }
  }

  use ast::BinaryOp::*;
  let (op_trt, op_sid) = match op {
    Add => (ctx.ideps.prim_op().unwrap().op_add, ctx.sin.sid_add()),
    Sub => (ctx.ideps.prim_op().unwrap().op_sub, ctx.sin.sid_sub()),
    Mul => (ctx.ideps.prim_op().unwrap().op_mul, ctx.sin.sid_mul()),
    Div => (ctx.ideps.prim_op().unwrap().op_div, ctx.sin.sid_div()),
    Rem => (ctx.ideps.prim_op().unwrap().op_rem, ctx.sin.sid_rem()),
    _ => unreachable!()
  };
  
  // Find Impl
  let (op_fun, op_ret) = Infer::lookup_method_with_trait(ctx, op_trt, lhs, rhs, op_sid, pos)
    .ok_or_else(||
      Message::error(X_NOT_IMPLEMENTED_FOR_TYPE
        .args(&[
          "trait",
          &format!("{}<{}>", ctx.type_name(op_trt), ctx.type_name(ctx.cre.get(rhs).ety)),
          &ctx.type_name(ctx.cre.get(lhs).ety),
        ]),
        Label::new_pos(pos)
      )
    )?;
  
  
  let args = ctx.cre.extra(&[lhs, rhs]);

  let it = hir::Expr {
    kind: hir::ExprKind::Call { callee: op_fun, args },
    category: ExprCategory::RValue,
    ety: op_ret,
  }.push(ctx.cre);

  Ok(it)
}

fn low_binary_condition(ctx: &mut Ctx, op: ast::BinaryOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<hir::ExprId, Message> {
  let lhs_ex = ctx.cre.get(lhs);
  let rhs_ex = ctx.cre.get(rhs);

  let lhs_ty = ctx.get(lhs_ex.ety);

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
    ety: ctx.prims.ty_bool,
  };
  
  Ok(ctx.cre.push(this))
}

fn low_binary_logic(ctx: &mut Ctx, op: ast::BinaryOp, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<hir::ExprId, Message> {
  let lhs_ex = ctx.cre.get(lhs);
  let rhs_ex = ctx.cre.get(rhs);

  let lhs_ty = ctx.get(lhs_ex.ety);

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
    ety: ctx.prims.ty_bool,
  };

  Ok(ctx.cre.push(this))
}


// Assign
pub fn low_assign(ctx: &mut Ctx, fctx: &FunCtx, lhs: ast::ExprId, rhs: ast::ExprId, op_span: Span) -> Result<hir::ExprId, Message> {
  let lhs_hir = ExprLow::low(ctx, fctx, lhs)?;
  let rhs_hir = ExprLow::low(ctx, fctx, rhs)?;
  err_bypass!(ctx, lhs_hir, rhs_hir);

  helper::verify_assignable(ctx, lhs, lhs_hir, op_span)?;

  let lhs_ty = ctx.cre.get(lhs_hir).ety;
  let rhs_ty = ctx.cre.get(rhs_hir).ety;

  if lhs_ty != rhs_ty {
    todo!("{:#?} != {:#?}", ctx.get(lhs_ty), ctx.get(rhs_ty))
  }

  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Assign {lhs: lhs_hir, rhs: rhs_hir},
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_unit,
  };
  
  Ok(ctx.cre.push(this))
}


// Assign Op
pub fn low_assign_op(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, op: ast::BinaryOp, lhs: ast::ExprId, rhs: ast::ExprId, op_span: Span) -> Result<hir::ExprId, Message> {
  let lhs_hir = ExprLow::low(ctx, fctx, lhs)?;
  let rhs_hir = ExprLow::low(ctx, fctx, rhs)?;
  err_bypass!(ctx, lhs_hir, rhs_hir);

  helper::verify_assignable(ctx, lhs, lhs_hir, op_span)?;

  let lhs_ty = ctx.cre.get(lhs_hir).ety;
  let rhs_ty = ctx.cre.get(rhs_hir).ety;

  if lhs_ty != rhs_ty {
    todo!("{:#?} != {:#?}", ctx.get(lhs_ty), ctx.get(rhs_ty))
  }


  // Op
  use ast::BinaryOp::*;
  
  let id = match op {
    // Arithmetic
    Add | Sub | Mul | Div | Rem => low_binary_arithmetic(ctx, it.pos, op, lhs_hir, rhs_hir)?,

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
pub fn low_call(ctx: &mut Ctx, fctx: &FunCtx, it: &ast::Expr, callee: ast::ExprId, args: ast::ExprRng) -> Result<hir::ExprId, Message> {
  let mut callee = ExprLow::low(ctx, fctx, callee)?;
  
  let this = if let hir::ExprKind::BoundSelfMethod { callee: my_callee, this } = ctx.cre.get(callee).kind {
    callee = my_callee;
    Some(this)
  } else { None };
  
  err_bypass!(ctx, callee);

  if let Some(this) = this {
    err_bypass!(ctx, this);
  }

  let mut user_args = Vec::with_capacity(args.count() as usize);
  for id in ctx.src.extra_get(args) {
    let id = ExprLow::low(ctx, fctx, id)?;
    err_bypass!(ctx, id);
    user_args.push(id);
  }

  // Target Function Selection via Infer
  if let Some(this) = this {
    callee = Infer::resolve_method(ctx, this, callee, &user_args, it.pos)?;
  }

  let callee_ety = ctx.cre.get(callee).ety;

  let hir::TypeKind::Fun{self_kind, args: trait_args, ret: trait_ret} = ctx.get(callee_ety).kind else {
    return Err(Message::error(X_NOT_IMPLEMENTED_FOR_TYPE.args(&["trait", "core::ops::Call", &ctx.type_name(callee_ety)]), Label::new_pos(it.pos)))
  };
  

  // Self Check
  let explicit_self = match (self_kind, this) {
    (None, None) => false,

    // Member Called
    (Some(kind), Some(this)) => {
      let this_ty = ctx.cre.get(this).ety;

      TypeMatch::matches_pos(ctx, kind, this_ty, it.pos)?;
      false
    }

    // Non Member
    (None, Some(..)) => {
      let fn_name = if let hir::ExprKind::GlobalRef(item) = ctx.cre.get(callee).kind { ctx.poit(item).to_string() } else { panic!() };
      return Err(Message::error(USE_ASSOCIATED_FUNCTION_SYNTAX_INSTEAD.args(&[&fn_name]), Label::new_pos(it.pos)));
    }

    (Some(..), None) => true
  };

  let args_hir = {
    let mut vec = vec![];

    if let Some(this) = this {
      vec.push(this);
    }

    vec.extend(&user_args);

    ctx.cre.extra(&vec)
  };


  let args_ct = user_args.len() as u32;

  let trait_ct = if explicit_self {trait_args.count()+1} else {trait_args.count()};


  if args_ct != trait_ct {
    let extra1 = if trait_ct > args_ct {
      Some(X_ARGUMENTS_ARE_MISSING.args(&[&(trait_ct-args_ct).to_string()]))
    } else {
      None
    };

    return Err(Message::error(FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED
      .args(&[
        &trait_ct.to_string(),
        &args_ct.to_string(),
      ]),
      Label::new_pos(it.pos))
      .add_if(extra1)
    )
  }

  if explicit_self {
    let self_arg = user_args[0];
    let self_pos = ctx.src.get(ctx.src.extra_get(args).next().unwrap()).pos;
    let self_ty = ctx.cre.get(self_arg).ety;
    TypeMatch::matches_pos(ctx, self_kind.unwrap(), self_ty, self_pos)?;

    for (&supp_hir, supp_ast, trt) in izip!(&user_args[1..], ctx.src.extra_get(args).skip(1), ctx.extra_get(trait_args)) {
      let hir::Thing::NamedType(_, trt_ty) = *ctx.get(trt) else { panic!() };
      
      let supp_pos = ctx.src.get(supp_ast).pos;
      let supp_ty = ctx.cre.get(supp_hir).ety;

      TypeMatch::matches_pos(ctx, trt_ty, supp_ty, supp_pos)?;
    }
  } else {
    for (&supp_hir, supp_ast, trt) in izip!(&user_args, ctx.src.extra_get(args), ctx.extra_get(trait_args)) {
      let hir::Thing::NamedType(_, trt_ty) = *ctx.get(trt) else { panic!() };
      
      let supp_pos = ctx.src.get(supp_ast).pos;
      let supp_ty = ctx.cre.get(supp_hir).ety;

      TypeMatch::matches_pos(ctx, trt_ty, supp_ty, supp_pos)?;
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
pub fn low_field_create(ctx: &mut Ctx, fctx: &FunCtx, lhs: ast::ExprId, fields: ast::ThingRng, brace_span: Span) -> Result<hir::ExprId, Message> {
  let lhs = ExprLow::low(ctx, fctx, lhs)?;
  
  let hir::ExprKind::TypeOf{kind} = ctx.cre.get(lhs).kind else {
    return Err(Message::error(ONLY_TYPES_CAN_BE_INITIALIZED_IN_THIS_WAY, Label::new_pos(brace_span)))
  };

  if matches!(ctx.get(kind).kind, hir::TypeKind::GenericRaw { .. }) {
    return Err(Message::error(CANNOT_USE_GENERIC_WITHOUT_SPECIALIZATION, Label::new_pos(brace_span)));
  }


  // Get Keys
  let (keys, sort) = {
    let hir::TypeKind::Struct(fields) = ctx.get(kind).kind else { panic!() };
    let krate = ctx.get_krate(kind.cid());
    
    let mut keys = FxHashMap::default();
    let mut sort = vec![];
    
    for id in krate.extra_get(fields) {
      let hir::Thing::NamedType(name, kind) = *krate.get(id) else { panic!() };
      
      keys.insert(name, kind);
      sort.push(name);
    }
    
    (keys, sort)
  };


  // Gived Keys
  let gived_keys = {
    let mut gived_keys = FxHashMap::default();

    for id in ctx.src.extra_get(fields) {
      let ast::ThingKind::NamedExpr(name, expr) = ctx.src.get(id).kind else { panic!() };
      
      let expr_pos = ctx.src.get(expr).pos;
      let expr = ExprLow::low(ctx, fctx, expr)?;
      let expr_ty = ctx.cre.get(expr).ety;

      if let Some(&kind) = keys.get(&name.sid()) {
        if kind != expr_ty {
          TypeMatch::matches_pos(ctx, kind, expr_ty, expr_pos)?;
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
pub fn low_member(ctx: &mut Ctx, fctx: &FunCtx, rng: ast::ExprRng) -> Result<hir::ExprId, Message> {
  let ids = ctx.src.extra_get(rng).collect_vec();
  let mut current_id = ExprLow::low(ctx, fctx, ids[0])?;

  for &id in &ids[1..] {
    err_bypass!(ctx, current_id);

    let ast::ExprKind::Nick(ident) = ctx.src.get(id).kind else { panic!("expected field identifier"); };

    
    // Basic Search
    let bsearch = match ctx.get(ctx.cre.get(current_id).ety).kind {
      hir::TypeKind::Struct(fields) => Some(low_member_struct(ctx, current_id, ident, fields)?),

      _ => None,
    }.flatten();

    if let Some(chain) = bsearch {
      current_id = chain; continue;
    }


    // Impl Search
    let isearch = Infer::lookup_method(ctx, current_id, ident.sid());

    if let Some(chain) = isearch {
      current_id = chain; continue;
    }


    return Err(Message::error(HAS_NO_FIELD_NAMED_X.args(&[ident.str(ctx.far)]), Label::new_pos(ident)));
  }

  Ok(current_id)
}

pub fn low_member_struct(ctx: &mut Ctx, chain: hir::ExprId, ident: Ident, fields: hir::ThingRng) -> Result<Option<hir::ExprId>, Message> {
  let krate = ctx.get_krate(ctx.cre.get(chain).ety.cid());

  let mut found = None;

  for (idx, id) in krate.extra_get(fields).enumerate() {
    let hir::Thing::NamedType(name, fty) = *krate.get(id) else { panic!() };
    if name == ident.sid() {
      found = Some((idx as u32, fty));
      break;
    }
  }
  
  let Some(found) = found else { return Ok(None); };

  let current = ctx.cre.get(chain);


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Field {
      target: chain,
      idx: found.0,
    },
    category: current.category,
    ety: found.1,
  };

  Ok(Some(ctx.cre.push(this)))
}
