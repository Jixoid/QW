use rustc_hash::FxHashMap;
use inkwell::values::{BasicValueEnum, PointerValue};
use qwc_mir::SSA;


pub struct FnCtx<'ctx> {
  pub stack_slots: Vec<PointerValue<'ctx>>,
  pub ssa_map: FxHashMap<SSA, BasicValueEnum<'ctx>>,
}


impl<'ctx> FnCtx<'ctx> {
  pub fn new() -> Self {
    Self {
      stack_slots: Vec::new(),
      ssa_map: FxHashMap::default(),
    }
  }
}
