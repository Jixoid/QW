use inkwell::values::{BasicValue, BasicValueEnum};
use qwc_mir::{Const, Value};

use crate::{context::{CGenCtx, SymbolVal}, fn_ctx::FnCtx};


pub struct ValueLow;

impl ValueLow {

  pub fn low<'ctx>(
    cgen: &CGenCtx<'ctx, '_>,
    fn_ctx: &FnCtx<'ctx>,
    it: &Value,
  ) -> BasicValueEnum<'ctx> {
    match it {
      Value::SSA(ssa) => {
        *fn_ctx.ssa_map.get(ssa).unwrap_or_else(|| {
          panic!("SSA register {:?} used before definition", ssa)
        })
      }

      Value::Const(c) => match c {
        Const::Unit => cgen.ctx.const_struct(&[], false).into(),
        Const::Bool(b) => cgen.ctx.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
        Const::Int(i) => cgen.ctx.i32_type().const_int(*i as u64, true).into(),
      },

      Value::GlobalRef(symb) => match cgen.symbols.get(symb).unwrap_or_else(|| {
        panic!("Global symbol {:?} not found in symbol table", symb)
      }) {
        SymbolVal::Global(gv) => gv.as_pointer_value().as_basic_value_enum(),
        SymbolVal::Function(fv) => fv.as_global_value().as_pointer_value().as_basic_value_enum(),
      },

      Value::StackRef(idx) => {
        fn_ctx.stack_slots[*idx as usize].as_basic_value_enum()
      }
    }
  }

}
