use std::path::Path;
use inkwell::{context::Context, module::Module};
use qwc_mir::Krate;
use qwc_cgen::ICGen;

mod context;
mod fn_ctx;
mod symb_p;
mod blok_p;
mod type_p;
mod value_p;

pub use context::{CGenCtx, SymbolVal};
pub use fn_ctx::FnCtx;
pub use symb_p::SymbLow;
pub use blok_p::BlokLow;
pub use type_p::{TypeLow, any_type_to_basic};
pub use value_p::ValueLow;


pub struct CGenLLVM;

impl ICGen for CGenLLVM {
  fn generate(cre: &Krate, fpath: &Path) -> Result<(), String> {
    let ctx = Context::create();
    let mol = Self::compile_to_module(&ctx, "main", cre);

    mol.verify().map_err(|err| err.to_string())?;

    mol.print_to_file(fpath).map_err(|err| err.to_string())?;
    
    Ok(())
  }
}



impl CGenLLVM {

  pub fn compile_to_module<'ctx>(ctx: &'ctx Context, module_name: &str, cre: &Krate) -> Module<'ctx> {
    let mol = ctx.create_module(module_name);
    let builder = ctx.create_builder();

    let mut cgen = CGenCtx::new(ctx, &mol, &builder, cre);

    // Pass 1: Declare all symbols (prototypes & globals)
    SymbLow::declare_symbols(&mut cgen);

    // Pass 2: Define all symbols (initializers & function bodies)
    SymbLow::define_symbols(&mut cgen);

    mol
  }

  pub fn generate_to_string(cre: &Krate) -> Result<String, String> {
    let ctx = Context::create();
    let mol = Self::compile_to_module(&ctx, "main", cre);

    mol.verify().map_err(|e| e.to_string())?;
    Ok(mol.print_to_string().to_string())
  }

  pub fn generate_to_file(cre: &Krate, path: &Path) -> Result<(), String> {
    let ctx = Context::create();
    let mol = Self::compile_to_module(&ctx, "main", cre);

    mol.verify().map_err(|e| e.to_string())?;
    mol.print_to_file(path).map_err(|e| e.to_string())?;
    Ok(())
  }

}
