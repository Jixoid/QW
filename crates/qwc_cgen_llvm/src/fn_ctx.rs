use rustc_hash::FxHashMap;
use inkwell::values::BasicValueEnum;
use qwc_mir::SSA;


pub struct FnCtx<'ctx> {
  pub params: Vec<BasicValueEnum<'ctx>>,
  pub ssa_map: FxHashMap<SSA, BasicValueEnum<'ctx>>,
}


impl<'ctx> FnCtx<'ctx> {
  pub fn new(params: Vec<BasicValueEnum<'ctx>>) -> Self {
    Self {
      params,
      ssa_map: FxHashMap::default(),
    }
  }
}

