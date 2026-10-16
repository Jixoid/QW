/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use inkwell::values::{AnyValue, AnyValueEnum, ArrayValue, BasicValue, BasicValueEnum};
use itertools::Itertools;
use qwc_mir::{Const, Value};
use rustc_apfloat::Float;

use crate::{FunCtx, cgen::{CtxI, CtxM, SymbolVal}, type_p::{TypeLow, any_type_to_basic}};


pub struct ValueLow;

impl ValueLow {

  pub fn low<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, fctx: &FunCtx<'ctx>, it: &Value) -> BasicValueEnum<'ctx> {
    match it {
      Value::SSA(ssa) => {
        *fctx.ssa_map.get(ssa).unwrap_or_else(|| {
          panic!("SSA register {:?} used before definition", ssa)
        })
      }

      Value::Const(c) => match *c {
        Const::Unit => ictx.ctx.const_struct(&[], false).into(),
        Const::Bool(b) => ictx.ctx.bool_type().const_int(if b { 1 } else { 0 }, false).into(),
        
        Const::Str(s) => ictx.ctx.const_string(ictx.sin.str(s).as_bytes(), false).into(),
        
        Const::I8(i) => ictx.ctx.i8_type().const_int(i as u64, true).into(),
        Const::I16(i) => ictx.ctx.i16_type().const_int(i as u64, true).into(),
        Const::I32(i) => ictx.ctx.i32_type().const_int(i as u64, true).into(),
        Const::I64(i) => ictx.ctx.i64_type().const_int(i as u64, true).into(),
        Const::I128(i) => ictx.ctx.i128_type().const_int_arbitrary_precision(&[i as u64, (i >> 64) as u64]).into(),
        
        
        Const::F16(fp) => ictx.builder.build_bit_cast(
          ictx.ctx.i16_type().const_int(fp.to_bits() as u16 as u64, false),
          ictx.ctx.f16_type(),
          ""
        ).unwrap(),
        
        Const::F32(fp) => ictx.builder.build_bit_cast(
          ictx.ctx.i32_type().const_int(fp.to_bits() as u32 as u64, false),
          ictx.ctx.f32_type(),
          ""
        ).unwrap(),
        
        Const::F64(fp) => ictx.builder.build_bit_cast(
          ictx.ctx.i64_type().const_int(fp.to_bits() as u64, false),
          ictx.ctx.f64_type(),
          ""
        ).unwrap(),
        
        Const::F128(fp) => {
          let bits = fp.to_bits();

          let (l,h) = (bits as u64, (bits >> 64) as u64);

          ictx.builder.build_bit_cast(
            ictx.ctx.i128_type().const_int_arbitrary_precision(&[l,h]),
            ictx.ctx.i128_type(),
            ""
          ).unwrap()
        }

        Const::Array {kind, rng} => {
          let elem_ty = any_type_to_basic(TypeLow::low(ictx, ictx.cre.get(kind)));
          
          let rng = ictx.cre.extra_get(rng).map(|id| {
            let value = ictx.cre.get(id);
            assert_matches!(value, Value::Const(..));
            
            ValueLow::low(uctx, ictx, fctx, value)
          }).collect_vec();
          
          let arr_val = unsafe { ArrayValue::new_const_array(&elem_ty, &rng) };

          arr_val.as_basic_value_enum()
        }
      },

      Value::GlobalRef(symb) => match uctx.symbols.get(symb).unwrap_or_else(|| {
        panic!("Global symbol {:?} not found in symbol table", symb)
      }) {
        SymbolVal::Global(gv) => gv.as_pointer_value().as_basic_value_enum(),
        SymbolVal::Function(fv) => fv.as_global_value().as_pointer_value().as_basic_value_enum(),
      },

      Value::Param(idx) => {
        fctx.params[*idx as usize]
      }
    }
  }

  pub fn low_fun<'ctx>(uctx: &mut CtxM<'ctx>, _ictx: &CtxI<'ctx, '_>, _fctx: &FunCtx<'ctx>, it: &Value) -> AnyValueEnum<'ctx> {
    match it {
      Value::GlobalRef(symb) => match uctx.symbols.get(symb).unwrap_or_else(|| {
        panic!("Global symbol {:?} not found in symbol table", symb)
      }) {
        SymbolVal::Global(gv) => gv.as_any_value_enum(),
        SymbolVal::Function(fv) => fv.as_any_value_enum(),
      },
      
      _ => unreachable!()
    }
  }

}
