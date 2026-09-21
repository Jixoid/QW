use rustc_hash::FxHashMap;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use qwc_mir::{SSA, TypeId};


pub struct FnCtx<'ctx> {
  pub func: FunctionValue<'ctx>,
  pub ret_ty_id: TypeId,
  pub is_ret_unit: bool,
  pub stack_slots: Vec<PointerValue<'ctx>>,
  pub ssa_map: FxHashMap<SSA, BasicValueEnum<'ctx>>,
}


impl<'ctx> FnCtx<'ctx> {
  pub fn new(func: FunctionValue<'ctx>, ret_ty_id: TypeId, is_ret_unit: bool) -> Self {
    Self {
      func,
      ret_ty_id,
      is_ret_unit,
      stack_slots: Vec::new(),
      ssa_map: FxHashMap::default(),
    }
  }
}
