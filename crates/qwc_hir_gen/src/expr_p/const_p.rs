/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::str::FromStr;

use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, Const, ExprCategory};
use qwc_string_interner::Sid;
use rustc_apfloat::ieee;

use crate::Ctx;


// ZST
pub fn low_unit(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Unit),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_unit,
  };

  Ok(ctx.cre.push(this))
}


// Primitive
pub fn low_bool(ctx: &mut Ctx, val: bool) -> Result<hir::ExprId, Message> {
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Bool(val)),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_bool,
  };

  Ok(ctx.cre.push(this))
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

pub fn low_number(ctx: &mut Ctx, span: Span) -> Result<hir::ExprId, Message> {
  let str = span.str(ctx.far);

  enum FP {
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
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
      
      "f16"  => FP::F16,
      "f32"  => FP::F32,
      "f64"  => FP::F64,
      "f128" => FP::F128,

      _ => return Err(Message::error(INVALID_SUFFIX_X_FOR_X_LITERAL.args(&[suffix, "int"]), Label::new_pos(span)))
    };

    (fp, begin)
  } else {
    (FP::I32, str)
  };

  let str: &str = &str.replace('_', "");

  let (ety, cons) = match fp {
    FP::U8   => (ctx.prims.ty_u8,   Const::U8(u8::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::U16  => (ctx.prims.ty_u16,  Const::U16(u16::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::U32  => (ctx.prims.ty_u32,  Const::U32(u32::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::U64  => (ctx.prims.ty_u64,  Const::U64(u64::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::U128 => (ctx.prims.ty_u128, Const::U128(u128::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),

    FP::I8   => (ctx.prims.ty_i8,   Const::I8(i8::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::I16  => (ctx.prims.ty_i16,  Const::I16(i16::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::I32  => (ctx.prims.ty_i32,  Const::I32(i32::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::I64  => (ctx.prims.ty_i64,  Const::I64(i64::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::I128 => (ctx.prims.ty_i128, Const::I128(i128::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    
    FP::F16  => (ctx.prims.ty_f16,  Const::F16(ieee::Half::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::F32  => (ctx.prims.ty_f32,  Const::F32(ieee::Single::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::F64  => (ctx.prims.ty_f64,  Const::F64(ieee::Double::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
    FP::F128 => (ctx.prims.ty_f128, Const::F128(ieee::Quad::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?)),
  };


  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(cons),
    category: ExprCategory::RValue,
    ety,
  };

  Ok(ctx.cre.push(this))
}

pub fn low_float(ctx: &mut Ctx, span: Span) -> Result<hir::ExprId, Message> {
  let str = span.str(ctx.far);

  enum FP { F16, F32, F64, F128 }

  let (fp, str) = if let Some((begin, suffix)) = extract_suffix(str) {
    let fp = match suffix {
      "f16"  => FP::F16,
      "f32"  => FP::F32,
      "f64"  => FP::F64,
      "f128" => FP::F128,

      _ => return Err(Message::error(INVALID_SUFFIX_X_FOR_X_LITERAL.args(&[suffix, "float"]), Label::new_pos(span)))
    };

    (fp, begin)
  } else {
    (FP::F32, str)
  };

  let str: &str = &str.replace('_', "");

  let (ety, cons) = match fp {
    FP::F16  => (ctx.prims.ty_f16,  Const::F16(ieee::Half::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_FLOAT, Label::new_pos(span)))?)),
    FP::F32  => (ctx.prims.ty_f32,  Const::F32(ieee::Single::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_FLOAT, Label::new_pos(span)))?)),
    FP::F64  => (ctx.prims.ty_f64,  Const::F64(ieee::Double::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_FLOAT, Label::new_pos(span)))?)),
    FP::F128 => (ctx.prims.ty_f128, Const::F128(ieee::Quad::from_str(str).map_err(|_| Message::error(CANNOT_CONVERT_TO_FLOAT, Label::new_pos(span)))?)),
  };


  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(cons),
    category: ExprCategory::RValue,
    ety,
  };

  Ok(ctx.cre.push(this))
}

pub fn low_string(ctx: &mut Ctx, sid: Sid) -> Result<hir::ExprId, Message> {
  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Str(sid)),
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_ref(ctx.cre, ctx.ideps, ctx.prims.ty_str, false),
  };

  Ok(ctx.cre.push(this))
}

pub fn low_char(ctx: &mut Ctx, span: Span, sid: Sid) -> Result<hir::ExprId, Message> {
  let str = ctx.sin.str(sid);
  let str = &str[..str.len() - 1];

  let c = str.parse::<char>().map_err(|_| Message::error(CANNOT_CONVERT_TO_CHAR, Label::new_pos(span)))?;


  // Post
  let this = hir::Expr{
    kind: hir::ExprKind::Const(Const::Char(c)),
    category: ExprCategory::RValue,
    ety: ctx.prims.ty_char,
  };

  Ok(ctx.cre.push(this))
}
