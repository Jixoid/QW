/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{fs, path::{Path, PathBuf}, process::Command};

use inkwell::{builder::Builder, context::Context, module::Module, types::BasicTypeEnum, values::{FunctionValue, GlobalValue}};
use inkwell::targets::{InitializationConfig, Target, TargetMachine, TargetTriple};
use qwc_cgen::{CodeModel, GenOptions, GenOutput, IBackend, ICodeGen, ILinkObj, IRun, LinkOptions, LinkTarget, Optimization, RelocMode};
use qwc_mir::{Krate, SymbId, TypeId};
use qwc_string_interner::StrInterner;
use rustc_hash::FxHashMap;

use crate::SymbLow;


pub struct GenLLVM;

impl IBackend for GenLLVM {
	fn code_gen(&self) -> Box<dyn ICodeGen> { Box::new(Self) }
	fn link_obj(&self) -> Box<dyn ILinkObj> { Box::new(Self) }
	fn run(&self) -> Option<Box<dyn IRun>> { Some(Box::new(Self)) }
}


impl ICodeGen for GenLLVM {
	fn generate(&self, cre: &Krate, sin: &StrInterner, outdir: &Path, options: &GenOptions) -> Result<GenOutput, String> {
		let ctx = Context::create();
		let mol = Self::compile_to_module(&ctx, "main", cre, sin);

		mol.verify().map_err(|err| err.to_string())?;

		// Out
		Target::initialize_all(&InitializationConfig::default());
		let triple = options.triple.clone().map(|str| TargetTriple::create(&str)).unwrap_or_else(TargetMachine::get_default_triple);
		
		let binary = Target::from_triple(&triple)
			.map_err(|err| format!("{} for {}", err.to_string(), triple))?
			.create_target_machine(
				&triple,
				"generic",
				"",
				opt_into(options.opt_level),
				reloc_into(options.reloc_mode.unwrap_or(RelocMode::PIC)),
				mcmodel_into(options.code_model.unwrap_or(CodeModel::Small)),
			)
			.ok_or_else(|| String::from("target machine cannot created"))?
			.write_to_memory_buffer(&mol, inkwell::targets::FileType::Object)
			.map_err(|err| err.to_string())?
			.as_slice()
			.to_vec();

		let text_ir = options.emit_ir.then(|| mol.print_to_string().to_string());
		
		fs::write(outdir.join("out.o"), &binary).map_err(|err| format!("io: {err}"))?;
		
		let ll = outdir.join("out.ll");
		
		if let Some(text_ir) = &text_ir {
			fs::write(ll, text_ir).map_err(|err| format!("io: {err}"))?;
		}
		else if text_ir.is_none() && ll.exists() {
			fs::remove_file(ll).map_err(|err| format!("io: {err}"))?;
		}

		Ok(GenOutput{
			binary,
			text_ir,
		})
	}
}

impl ILinkObj for GenLLVM {
	fn link(&self, target: &Path, out: &Path, deps: &[&Path], args: &[&str], options: &LinkOptions) -> Result<(), String> {
		let mut cmd = Command::new("ld.lld");

		// target / reloc mode
		match options.target {
			Some(LinkTarget::Shared) => {
				cmd.arg("-shared");
			}
			_ => {
				if let Some(reloc_mode) = &options.reloc_mode {
					cmd.arg(match reloc_mode {
						RelocMode::PIC => "--pie",
						RelocMode::Static => "--static",
					});
				}

				if options.is_dynamic {
					cmd.arg("-dynamic-linker").arg("/lib64/ld-linux-x86-64.so.2");
				}
			}
		}

		// code model
		if let Some(code_model) = &options.code_model {
			cmd.arg("-mllvm").arg(format!("-code-model={}", match code_model {
				CodeModel::Kernel => "kernel",
				CodeModel::Small  => "small",
				CodeModel::Medium => "medium",
				CodeModel::Large  => "large",
			}));
		}

		// lto
		if options.lto {
			cmd.arg("--lto-O2");
		}

		// strip
		if options.strip {
			cmd.arg("--strip-debug");
		}
		
		// main
		cmd.arg(target.join("out.o"));

		// out
		cmd.arg("-o").arg(match options.target.unwrap() {
			LinkTarget::Shared => out.join("out.so"),
			LinkTarget::Exec   => out.join("out"),
		});
		
		// deps
		for &dep in deps {
			cmd.arg(PathBuf::from(dep).join("out.o"));
		}

		for &arg in args {
			cmd.arg(arg);
		}


		// Linker
		let out = cmd.output().map_err(|err| format!("linker: {err}"))?;

		if !out.status.success() {
			eprint!("{}", String::from_utf8_lossy(&out.stderr));
			eprint!("{}", String::from_utf8_lossy(&out.stdout));

			return Err(format!("linker return error"));
		}

		Ok(())
	}
}

impl IRun for GenLLVM {
	fn run(&self, target: &Path, args: &[&str]) -> Result<i32, String> {
		let exe = target.join("out");

		let mut cmd = Command::new(exe);
		cmd.args(args);

		let out = cmd.output().map_err(|err| format!("program: {err}"))?;

		if !out.status.success() {
			eprint!("{}", String::from_utf8_lossy(&out.stderr));
			eprint!("{}", String::from_utf8_lossy(&out.stdout));

			return Err(format!("linker return error"));
		}
		
		Ok(out.status.code().unwrap_or(1))
	}
}



#[derive(Clone, Copy, Debug)]
pub enum SymbolVal<'ctx> {
	Global(GlobalValue<'ctx>),
	Function(FunctionValue<'ctx>),
}

pub struct CtxI<'ctx, 'a> {
	pub ctx: &'ctx Context,
	pub mol: &'a Module<'ctx>,
	pub sin: &'a StrInterner,
	pub builder: &'a Builder<'ctx>,
	pub cre: &'a Krate,
}

pub struct CtxM<'ctx> {
	pub type_cache: FxHashMap<TypeId, BasicTypeEnum<'ctx>>,
	pub symbols: FxHashMap<SymbId, SymbolVal<'ctx>>,
}

impl GenLLVM {
	pub fn compile_to_module<'ctx>(ctx: &'ctx Context, module_name: &str, cre: &Krate, sin: &StrInterner) -> Module<'ctx> {
		let mol = ctx.create_module(module_name);
		let builder = ctx.create_builder();

		let ictx = CtxI {ctx, mol: &mol, builder: &builder, cre, sin };
		let mut uctx = CtxM {type_cache: FxHashMap::default(),symbols: FxHashMap::default()};

		// Pass 1: Declare all symbols (prototypes & globals)
		SymbLow::declare_symbols(&mut uctx, &ictx);

		// Pass 2: Define all symbols (initializers & function bodies)
		SymbLow::define_symbols(&mut uctx, &ictx);

		mol
	}

	pub fn generate_to_string(cre: &Krate, sin: &StrInterner) -> Result<String, String> {
		let ctx = Context::create();
		let mol = Self::compile_to_module(&ctx, "main", cre, sin);

		mol.verify().map_err(|e| e.to_string())?;
		Ok(mol.print_to_string().to_string())
	}

	pub fn generate_to_file(cre: &Krate, sin: &StrInterner, path: &Path) -> Result<(), String> {
		let ctx = Context::create();
		let mol = Self::compile_to_module(&ctx, "main", cre, sin);

		mol.verify().map_err(|e| e.to_string())?;
		mol.print_to_file(path).map_err(|e| e.to_string())?;
		Ok(())
	}
}


fn mcmodel_into(it: CodeModel) -> inkwell::targets::CodeModel {
	match it {
		CodeModel::Small  => inkwell::targets::CodeModel::Small,
		CodeModel::Medium => inkwell::targets::CodeModel::Medium,
		CodeModel::Large  => inkwell::targets::CodeModel::Large,
		CodeModel::Kernel => inkwell::targets::CodeModel::Kernel,
	}
}

fn reloc_into(it: RelocMode) -> inkwell::targets::RelocMode {
	match it {
		RelocMode::PIC    => inkwell::targets::RelocMode::PIC,
		RelocMode::Static => inkwell::targets::RelocMode::Static,
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
