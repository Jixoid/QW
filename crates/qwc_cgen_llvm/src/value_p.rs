/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use inkwell::values::{AnyValue, AnyValueEnum, BasicValue, BasicValueEnum};
use qwc_mir::{Const, Value};

use crate::{FnCtx, cgen::{CtxI, CtxM, SymbolVal}};


pub struct ValueLow;

impl ValueLow {

  pub fn low<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>, fctx: &FnCtx<'ctx>, it: &Value) -> BasicValueEnum<'ctx> {
    match it {
      Value::SSA(ssa) => {
        *fctx.ssa_map.get(ssa).unwrap_or_else(|| {
          panic!("SSA register {:?} used before definition", ssa)
        })
      }

      Value::Const(c) => match c {
        Const::Unit => ictx.ctx.const_struct(&[], false).into(),
        Const::Bool(b) => ictx.ctx.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
        Const::Int(i) => ictx.ctx.i32_type().const_int(*i as u64, true).into(),
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

  pub fn low_fun<'ctx>(uctx: &mut CtxM<'ctx>, _ictx: &CtxI<'ctx, '_>, _fctx: &FnCtx<'ctx>, it: &Value) -> AnyValueEnum<'ctx> {
    match it {
      Value::GlobalRef(symb) => match uctx.symbols.get(symb).unwrap_or_else(|| {
        panic!("Global symbol {:?} not found in symbol table", symb)
      }) {
        SymbolVal::Global(gv) => gv.as_any_value_enum(),
        SymbolVal::Function(fv) => fv.as_any_value_enum(),
      },
      
      _ => panic!()
    }
  }

}
