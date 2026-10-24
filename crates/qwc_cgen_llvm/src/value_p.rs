/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use inkwell::{llvm_sys::core::LLVMConstBitCast, types::AsTypeRef, values::{AnyValue, AnyValueEnum, ArrayValue, AsValueRef, BasicValue, BasicValueEnum, FloatValue}};
use itertools::Itertools;
use qwc_mir::{Const, SymbId, Value};
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

      Value::Const(cons) => Self::low_const(ictx, cons),

      Value::GlobalRef(symb) => Self::low_global_ref(uctx, *symb),

      Value::Param(idx) => {
        fctx.params[*idx as usize]
      }
    }
  }

  pub fn low_const<'ctx>(ictx: &CtxI<'ctx, '_>, cons: &Const) -> BasicValueEnum<'ctx> {
    match *cons {
      Const::Unit => ictx.ctx.const_struct(&[], false).into(),
      Const::Bool(b) => ictx.ctx.bool_type().const_int(if b { 1 } else { 0 }, false).into(),
      
      Const::Str(s) => ictx.ctx.const_string(ictx.sin.str(s).as_bytes(), false).into(),
      
      Const::I8(i) => ictx.ctx.i8_type().const_int(i as u64, true).into(),
      Const::I16(i) => ictx.ctx.i16_type().const_int(i as u64, true).into(),
      Const::I32(i) => ictx.ctx.i32_type().const_int(i as u64, true).into(),
      Const::I64(i) => ictx.ctx.i64_type().const_int(i, true).into(),
      Const::I128(i) => ictx.ctx.i128_type().const_int_arbitrary_precision(&[i as u64, (i >> 64) as u64]).into(),
      
      
      Const::F16(fp) => {
        unsafe {
          let val_ref = LLVMConstBitCast(
            ictx.ctx.i16_type().const_int(fp.to_bits() as u16 as u64, false).as_value_ref(),
            ictx.ctx.f16_type().as_type_ref(),
          );

          std::mem::transmute::<_, FloatValue<'ctx>>(val_ref)
        }
        .as_basic_value_enum()
      }
      
      Const::F32(fp) => {
        unsafe {
          let val_ref = LLVMConstBitCast(
            ictx.ctx.i32_type().const_int(fp.to_bits() as u32 as u64, false).as_value_ref(),
            ictx.ctx.f32_type().as_type_ref(),
          );
        
          std::mem::transmute::<_, FloatValue<'ctx>>(val_ref)
        }
        .as_basic_value_enum()
      }
      
      Const::F64(fp) => {
        unsafe {
          let val_ref = LLVMConstBitCast(
            ictx.ctx.i64_type().const_int(fp.to_bits() as u64, false).as_value_ref(),
            ictx.ctx.f64_type().as_type_ref(),
          );

          std::mem::transmute::<_, FloatValue<'ctx>>(val_ref)
        }
        .as_basic_value_enum()
      }
      
      Const::F128(fp) => {
        let bits = fp.to_bits();
        let (l,h) = (bits as u64, (bits >> 64) as u64);

        unsafe {
          let val_ref = LLVMConstBitCast(
            ictx.ctx.i128_type().const_int_arbitrary_precision(&[l,h]).as_value_ref(),
            ictx.ctx.f128_type().as_type_ref(),
          );

          std::mem::transmute::<_, FloatValue<'ctx>>(val_ref)
        }
        .as_basic_value_enum()
      }
      
      Const::Array {kind, rng} => {
        let elem_ty = any_type_to_basic(TypeLow::low(ictx, ictx.cre.get(kind)));
        
        let rng = ictx.cre.extra_get(rng).map(|id| {
          let Value::Const(cons) = ictx.cre.get(id) else { unreachable!() };
          
          ValueLow::low_const(ictx, cons)
        }).collect_vec();
        
        let arr_val = unsafe { ArrayValue::new_const_array(&elem_ty, &rng) };

        arr_val.as_basic_value_enum()
      }
    }
  }

  pub fn low_global_ref<'ctx>(uctx: &mut CtxM<'ctx>, symb: SymbId) -> BasicValueEnum<'ctx> {
    match uctx.symbols.get(&symb).unwrap_or_else(|| {
      panic!("Global symbol {:?} not found in symbol table", symb)
    }) {
      SymbolVal::Global(gv) => gv.as_pointer_value().as_basic_value_enum(),
      SymbolVal::Function(fv) => fv.as_global_value().as_pointer_value().as_basic_value_enum(),
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
