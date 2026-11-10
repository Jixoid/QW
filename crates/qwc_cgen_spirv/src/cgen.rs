/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{fs, path::Path};

use qwc_cgen::{GenOptions, GenOutput, IBackend, ICodeGen, ILinkObj, IRun, LinkOptions};
use qwc_mir as mir;
use qwc_string_interner::StrInterner;
use rspirv::binary::{Assemble, Disassemble};
use rspirv::dr::Builder;
use rspirv::spirv::{AddressingModel, Capability, MemoryModel};
use rustc_hash::FxHashMap;

use crate::{symb_p::SymbLow, type_p::TypeLow};


pub struct GenSpirV;

impl IBackend for GenSpirV {
  fn code_gen(&self) -> Box<dyn ICodeGen> { Box::new(Self) }
  fn link_obj(&self) -> Box<dyn ILinkObj> { Box::new(Self) }
  fn run(&self) -> Option<Box<dyn IRun>> { None }
}

impl ILinkObj for GenSpirV {
  fn link(&self, _target: &Path, _out: &Path, _deps: &[&Path], _args: &[&str], _options: &LinkOptions) -> Result<(), String> {
    Ok(())
  }
}

impl ICodeGen for GenSpirV {
  fn generate(&self, mir: &mir::Krate, sin: &StrInterner, outdir: &Path, options: &GenOptions) -> Result<GenOutput, String> {
    let mut b = Builder::new();

    // Capabilities
    b.capability(Capability::Shader);
    b.capability(Capability::Linkage);
    b.capability(Capability::Int64);
    b.capability(Capability::Float64);

    // Memory Model
    b.memory_model(AddressingModel::Logical, MemoryModel::GLSL450);

    // Types and Symbols
    let mut ty_low = TypeLow::new();
    let mut symbols = FxHashMap::default();
    let mut entry_points = Vec::new();

    // Pass 1: Declare symbols
    SymbLow::declare_symbols(&mut b, mir, sin, &mut ty_low, &mut symbols, &mut entry_points)?;

    // Pass 2: Define symbols (function bodies)
    SymbLow::define_symbols(&mut b, mir, &mut ty_low, &symbols)?;

    // Register Entry Points
    SymbLow::register_entry_points(&mut b, &entry_points);

    // Assemble and Disassemble
    let module = b.module();
    let text_ir = options.emit_ir.then(|| module.disassemble());
    let words = module.assemble();
    let binary: Vec<u8> = words.into_iter().flat_map(|w| w.to_le_bytes()).collect();

    // Write binary out.spv
    fs::write(outdir.join("out.spv"), &binary).map_err(|err| format!("io: {err}"))?;

    // Optionally write text disassembly out.spvasm
    let spvasm_path = outdir.join("out.spvasm");
    if let Some(text) = &text_ir {
      fs::write(&spvasm_path, text).map_err(|err| format!("io: {err}"))?;
    } else if spvasm_path.exists() {
      let _ = fs::remove_file(spvasm_path);
    }

    Ok(GenOutput {
      binary,
      text_ir,
    })
  }
}
