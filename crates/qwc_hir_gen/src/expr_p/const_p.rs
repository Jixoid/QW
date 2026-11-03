/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::str::FromStr;

use qwc_diagnostic::{Label, Message, Span, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, Const, ExprCategory, Layout, PushOkApi, TypeAttr};
use qwc_ast as ast;
use qwc_string_interner::Sid;
use rustc_apfloat::ieee;

use crate::{Ctx, ExprLow, FunCtx, TypeMatch};


// ZST
pub fn low_unit(ctx: &mut Ctx, it: &ast::Expr) -> Status<hir::ExprId> {
  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Const(Const::Unit),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ctx.prims.ty_unit,
  };

  Status::Success(ctx.cre.push(this))
}


// Primitive
pub fn low_bool(ctx: &mut Ctx, it: &ast::Expr, val: bool) -> Status<hir::ExprId> {
  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Const(Const::Bool(val)),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ctx.prims.ty_bool,
  };

  Status::Success(ctx.cre.push(this))
}


fn extract_suffix(s: &str) -> Option<(&str, &str)> {
  let (begin, suffix) = s.rsplit_once('_')?;
  let first_char = suffix.chars().next()?;

  if !first_char.is_ascii_digit() {
    Some((begin, suffix))
  } else {
    None
  }
}

pub fn low_number(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, span: Span) -> Status<hir::ExprId> {
  let str = span.str(ctx.far);

  enum FP {
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    B8, B16, B32, B64, B128,
    F16, F32, F64, F128,
  }

  let (fp, str) = if let Some((begin, suffix)) = extract_suffix(str) {
    let fp = match suffix {
      "u8"   => FP::U8,
      "u16"  => FP::U16,
      "u32"  => FP::U32,
      "u64"  => FP::U64,
      "u128" => FP::U128,

      "i8"   => FP::I8,
      "i16"  => FP::I16,
      "i32"  => FP::I32,
      "i64"  => FP::I64,
      "i128" => FP::I128,

      "b8"   => FP::B8,
      "b16"  => FP::B16,
      "b32"  => FP::B32,
      "b64"  => FP::B64,
      "b128" => FP::B128,
      
      "f16"  => FP::F16,
      "f32"  => FP::F32,
      "f64"  => FP::F64,
      "f128" => FP::F128,

      _ => ret_err!(sum, Message::error(INVALID_SUFFIX_X_FOR_X_LITERAL.args(&[suffix, "int"]), Label::new_pos(span)))
    };

    (fp, begin)
  } else {
    (FP::I32, str)
  };

  let str: &str = &str.replace('_', "");

  macro_rules! parse_int {
    ($ty:ident, $cons:ident) => {
      match $ty::from_str(str) {
        Ok(v) => Const::$cons(v),
        Err(_) => ret_err!(sum, Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span))),
      }
    };
  }
  
  macro_rules! parse_bit {
    ($ty:ident, $cons:ident) => {
      match $ty::from_str(str) {
        Ok(v) => Const::$cons(v),
        Err(_) => ret_err!(sum, Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span))),
      }
    };
  }

  macro_rules! parse_float {
    ($ty:path, $cons:ident) => {
      match <$ty>::from_str(str) {
        Ok(v) => Const::$cons(v),
        Err(_) => ret_err!(sum, Message::error(CANNOT_CONVERT_TO_FLOAT, Label::new_pos(span))),
      }
    };
  }

  let (ety, cons) = match fp {
    FP::U8   => (ctx.prims.ty_u8,   parse_int!(u8, U8)),
    FP::U16  => (ctx.prims.ty_u16,  parse_int!(u16, U16)),
    FP::U32  => (ctx.prims.ty_u32,  parse_int!(u32, U32)),
    FP::U64  => (ctx.prims.ty_u64,  parse_int!(u64, U64)),
    FP::U128 => (ctx.prims.ty_u128, parse_int!(u128, U128)),

    FP::I8   => (ctx.prims.ty_i8,   parse_int!(i8, I8)),
    FP::I16  => (ctx.prims.ty_i16,  parse_int!(i16, I16)),
    FP::I32  => (ctx.prims.ty_i32,  parse_int!(i32, I32)),
    FP::I64  => (ctx.prims.ty_i64,  parse_int!(i64, I64)),
    FP::I128 => (ctx.prims.ty_i128, parse_int!(i128, I128)),

    FP::B8   => (ctx.prims.ty_b8,   parse_bit!(u8, B8)),
    FP::B16  => (ctx.prims.ty_b16,  parse_bit!(u16, B16)),
    FP::B32  => (ctx.prims.ty_b32,  parse_bit!(u32, B32)),
    FP::B64  => (ctx.prims.ty_b64,  parse_bit!(u64, B64)),
    FP::B128 => (ctx.prims.ty_b128, parse_bit!(u128, B128)),
    
    FP::F16  => (ctx.prims.ty_f16,  parse_float!(ieee::Half, F16)),
    FP::F32  => (ctx.prims.ty_f32,  parse_float!(ieee::Single, F32)),
    FP::F64  => (ctx.prims.ty_f64,  parse_float!(ieee::Double, F64)),
    FP::F128 => (ctx.prims.ty_f128, parse_float!(ieee::Quad, F128)),
  };


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Const(cons),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety,
  };

  Status::Success(ctx.cre.push(this))
}

pub fn low_float(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, span: Span) -> Status<hir::ExprId> {
  let str = span.str(ctx.far);

  enum FP { F16, F32, F64, F128 }

  let (fp, str) = if let Some((begin, suffix)) = extract_suffix(str) {
    let fp = match suffix {
      "f16"  => FP::F16,
      "f32"  => FP::F32,
      "f64"  => FP::F64,
      "f128" => FP::F128,

      _ => ret_err!(sum, Message::error(INVALID_SUFFIX_X_FOR_X_LITERAL.args(&[suffix, "float"]), Label::new_pos(span)))
    };

    (fp, begin)
  } else {
    (FP::F32, str)
  };

  let str: &str = &str.replace('_', "");

  macro_rules! parse_float {
    ($ty:path, $cons:ident) => {
      match <$ty>::from_str(str) {
        Ok(v) => Const::$cons(v),
        Err(_) => ret_err!(sum, Message::error(CANNOT_CONVERT_TO_FLOAT, Label::new_pos(span))),
      }
    };
  }

  let (ety, cons) = match fp {
    FP::F16  => (ctx.prims.ty_f16,  parse_float!(ieee::Half, F16)),
    FP::F32  => (ctx.prims.ty_f32,  parse_float!(ieee::Single, F32)),
    FP::F64  => (ctx.prims.ty_f64,  parse_float!(ieee::Double, F64)),
    FP::F128 => (ctx.prims.ty_f128, parse_float!(ieee::Quad, F128)),
  };


  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(cons),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety,
  };

  Status::Success(ctx.cre.push(this))
}

pub fn low_string(ctx: &mut Ctx, it: &ast::Expr, sid: Sid) -> Status<hir::ExprId> {
  let ety = ctx.tin.ty_ref(ctx.cre, ctx.prims.ty_str, false, None);
  
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Str(sid)),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety,
  };

  Status::Success(ctx.cre.push(this))
}

pub fn low_char(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, span: Span, sid: Sid) -> Status<hir::ExprId> {
  let str = ctx.sin.str(sid);
  let str = &str[..str.len() - 1];

  let c = match str.parse::<char>() {
    Ok(c) => c,
    Err(_) => ret_err!(sum, Message::error(CANNOT_CONVERT_TO_CHAR, Label::new_pos(span))),
  };


  // Post
  let this = hir::Expr {
    kind: hir::ExprKind::Const(Const::Char(c)),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ctx.prims.ty_char,
  };

  Status::Success(ctx.cre.push(this))
}


// Array
pub fn low_array(ctx: &mut Ctx, sum: &mut Summary, fctx: &FunCtx, it: &ast::Expr, args: ast::ExprRng) -> Status<hir::ExprId> {
  let (items_rng, kind) = {
    let mut items = vec![];
    let mut rety: Option<hir::TypeId> = None;
    
    for rid in ctx.src.extra_get(args) {
      let id = ExprLow::low(ctx, fctx, sum, rid)?;
      let ty = ctx.cre.get(id).ety;
      
      if let Some(expected_ty) = rety {
        TypeMatch::matches(ctx, sum, expected_ty, ty, ctx.get(id).pos, None)?;
      } else {
        rety = Some(ty);
      }

      items.push(id);
    }
    
    (ctx.cre.extra(&items), rety.unwrap_or(ctx.prims.ty_unit))
  };

  let len = hir::Expr {
    kind: hir::ExprKind::Const(Const::U32(items_rng.count())),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ctx.prims.ty_usize,
  }.push(ctx.cre);

  let ety = hir::Type {
    kind: hir::TypeKind::Array(kind, len),
    attr: TypeAttr::empty(),
    layout: Layout::STATIC,
  }.push(ctx.cre);

  hir::Expr {
    kind: hir::ExprKind::Const(Const::Array(kind, items_rng)),
    category: ExprCategory::RValue,
    pos: it.pos,
    ety,
  }.push_ok(ctx.cre)
}
