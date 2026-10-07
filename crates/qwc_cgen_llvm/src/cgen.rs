/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;

use inkwell::{
    builder::Builder, context::Context, memory_buffer::MemoryBuffer, module::Module, targets::{InitializationConfig, Target, TargetMachine, TargetTriple}, types::BasicTypeEnum, values::{FunctionValue, GlobalValue},
};
use qwc_cgen::{ICGen, Optimization, OutKind};
use qwc_mir::{Krate, SymbId, TypeId};
use rustc_hash::FxHashMap;

use crate::SymbLow;

#[derive(Clone, Copy, Debug)]
pub enum SymbolVal<'ctx> {
	Global(GlobalValue<'ctx>),
	Function(FunctionValue<'ctx>),
}

pub struct CtxI<'ctx, 'a> {
	pub ctx: &'ctx Context,
	pub mol: &'a Module<'ctx>,
	pub builder: &'a Builder<'ctx>,
	pub cre: &'a Krate,
}

pub struct CtxM<'ctx> {
	pub type_cache: FxHashMap<TypeId, BasicTypeEnum<'ctx>>,
	pub symbols: FxHashMap<SymbId, SymbolVal<'ctx>>,
}

pub struct CGenLLVM;

impl ICGen for CGenLLVM {
	fn generate(&self, cre: &Krate, ext_ll: bool, outk: OutKind, triple: &Option<String>, opt: Optimization) -> Result<(Vec<u8>, Option<String>), String> {
		let ctx = Context::create();
		let mol = Self::compile_to_module(&ctx, "main", cre);

		mol.verify().map_err(|err| err.to_string())?;

		// Out
		let out = match outk {
			OutKind::ByteCode => {
				mol.write_bitcode_to_memory().as_slice().to_vec()
			}

			OutKind::Object => {
				Target::initialize_all(&InitializationConfig::default());
				let triple = triple.clone().map(|str| TargetTriple::create(&str)).unwrap_or_else(|| TargetMachine::get_default_triple());
				
				Target::from_triple(&triple)
				.map_err(|err| format!("{} for {}", err.to_string(), triple))?
				.create_target_machine(
					&triple,
					"generic",
					"",
					opt_into(opt),
					inkwell::targets::RelocMode::PIC,
					inkwell::targets::CodeModel::Default,
				)
				.ok_or_else(|| String::from("target machine cannot created"))?
				.write_to_memory_buffer(&mol, inkwell::targets::FileType::Object)
				.map_err(|err| err.to_string())?
				.as_slice()
				.to_vec()
			}
		};
		

		let ll = ext_ll.then(|| mol.print_to_string().to_string());

		Ok((out, ll))
	}

	fn run_vm(&self, code: &[u8]) -> Result<i32, String> {
		let ctx = Context::create();

		let mol = Module
			::parse_bitcode_from_buffer(&MemoryBuffer::create_from_memory_range(code, "main"), &ctx)
			.map_err(|err| err.to_string())?;
	
		let execution_engine = mol
			.create_jit_execution_engine(inkwell::OptimizationLevel::Default)
			.map_err(|e| e.to_string())?;

		type QwEntryFunc = unsafe extern "C" fn();

    let qw_entry_jit_fn = unsafe {
			execution_engine.get_function::<QwEntryFunc>("qw_entry")
    };

		match qw_entry_jit_fn {
			Ok(func) => {unsafe { func.call(); }; Ok(0)},

			Err(err) => Err(err.to_string())
    }
	}
}

impl CGenLLVM {
	pub fn compile_to_module<'ctx>(ctx: &'ctx Context, module_name: &str, cre: &Krate) -> Module<'ctx> {
		let mol = ctx.create_module(module_name);
		let builder = ctx.create_builder();

		let ictx = CtxI {ctx, mol: &mol, builder: &builder, cre };
		let mut uctx = CtxM {type_cache: FxHashMap::default(),symbols: FxHashMap::default()};

		// Pass 1: Declare all symbols (prototypes & globals)
		SymbLow::declare_symbols(&mut uctx, &ictx);

		// Pass 2: Define all symbols (initializers & function bodies)
		SymbLow::define_symbols(&mut uctx, &ictx);

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


fn opt_into(it: Optimization) -> inkwell::OptimizationLevel {
	match it {
		Optimization::None       => inkwell::OptimizationLevel::None,
		Optimization::Less       => inkwell::OptimizationLevel::Less,
		Optimization::Default    => inkwell::OptimizationLevel::Default,
		Optimization::Aggressive => inkwell::OptimizationLevel::Aggressive,
	}
}
