use rustc_hash::FxHashMap;
use inkwell::{builder::Builder, context::Context, module::Module, types::BasicTypeEnum, values::{FunctionValue, GlobalValue}};
use qwc_mir::{Krate, SymbId, TypeId};


#[derive(Clone, Copy, Debug)]
pub enum SymbolVal<'ctx> {
  Global(GlobalValue<'ctx>),
  Function(FunctionValue<'ctx>),
}


pub struct CGenCtx<'ctx, 'a> {
  pub ctx: &'ctx Context,
  pub mol: &'a Module<'ctx>,
  pub builder: &'a Builder<'ctx>,
  pub cre: &'a Krate,
  pub type_cache: FxHashMap<TypeId, BasicTypeEnum<'ctx>>,
  pub symbols: FxHashMap<SymbId, SymbolVal<'ctx>>,
}


impl<'ctx, 'a> CGenCtx<'ctx, 'a> {
  pub fn new(ctx: &'ctx Context, mol: &'a Module<'ctx>, builder: &'a Builder<'ctx>, cre: &'a Krate) -> Self {
    Self {
      ctx,
      mol,
      builder,
      cre,
      type_cache: FxHashMap::default(),
      symbols: FxHashMap::default(),
    }
  }
}
