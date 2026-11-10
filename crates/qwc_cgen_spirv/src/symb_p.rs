/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir::{self as mir, SymbolKind, TypeKind};
use qwc_string_interner::StrInterner;
use rspirv::dr::Builder;
use rspirv::spirv::{ExecutionModel, FunctionControl, StorageClass, Word};
use rustc_hash::FxHashMap;

use crate::{inst_p::InstLow, type_p::TypeLow, value_p::FunCtx};


pub struct EntryPointInfo {
  pub id: Word,
  pub name: String,
}

pub struct SymbLow;

impl SymbLow {
  pub fn declare_symbols(
    b: &mut Builder,
    cre: &mir::Krate,
    _sin: &StrInterner,
    _ty_low: &mut TypeLow,
    symbols: &mut FxHashMap<mir::SymbId, Word>,
    entry_points: &mut Vec<EntryPointInfo>,
  ) -> Result<(), String> {
    for idx in 0..cre.symbols_len() {
      let sym_id = mir::SymbId::from_any(mir::AnyId::new(idx as u32, mir::id::NodeKind::Symb));
      let sym = cre.get(sym_id);
      let name = cre.sym_str(sym.name);

      match sym.kind {
        SymbolKind::Function { .. } => {
          let fn_id = b.id();
          symbols.insert(sym_id, fn_id);

          // Check if this is an entry point (e.g. main or qw_entry)
          let is_entry = name == "main" || name == "qw_entry";
          if is_entry {
            let entry_name = if name == "qw_entry" { "main".to_string() } else { name.to_string() };
            entry_points.push(EntryPointInfo { id: fn_id, name: entry_name });
          }
        }

        SymbolKind::Variable { value: _, ism: _ } => {
          let var_id = b.id();
          symbols.insert(sym_id, var_id);
        }

        SymbolKind::Vmt { .. } => {
          // VMT not supported in SPIR-V
        }
      }
    }

    Ok(())
  }

  pub fn define_symbols(
    b: &mut Builder,
    cre: &mir::Krate,
    ty_low: &mut TypeLow,
    symbols: &FxHashMap<mir::SymbId, Word>,
  ) -> Result<(), String> {
    for idx in 0..cre.symbols_len() {
      let sym_id = mir::SymbId::from_any(mir::AnyId::new(idx as u32, mir::id::NodeKind::Symb));
      let sym = cre.get(sym_id);

      match sym.kind {
        SymbolKind::Function { entry, blocks } => {
          let fn_id = symbols[&sym_id];
          let fun_ty = cre.get(sym.ety);
          let TypeKind::Fun { args, ret } = fun_ty.kind else {
            return Err(format!("expected function type for symbol {:?}", sym_id));
          };

          let ret_w = ty_low.low(b, cre, ret)?;
          let fn_ty_w = ty_low.low(b, cre, sym.ety)?;

          b.begin_function(ret_w, Some(fn_id), FunctionControl::NONE, fn_ty_w)
            .map_err(|e| e.to_string())?;

          let mut fctx = FunCtx::new();

          // Parameters
          for arg_id in cre.extra_get(args) {
            let arg_w = ty_low.low(b, cre, arg_id)?;
            let param_w = b.function_parameter(arg_w).map_err(|e| e.to_string())?;
            fctx.param_map.push(param_w);
          }

          // Pre-allocate block labels
          let mut block_map = FxHashMap::default();
          block_map.insert(entry, b.id());
          for blk_id in cre.extra_get(blocks) {
            block_map.entry(blk_id).or_insert_with(|| b.id());
          }

          // Process entry block first, then the remaining blocks
          let mut all_blocks = vec![entry];
          for blk_id in cre.extra_get(blocks) {
            if blk_id != entry && !all_blocks.contains(&blk_id) {
              all_blocks.push(blk_id);
            }
          }

          for blk_id in all_blocks {
            let label_id = block_map[&blk_id];
            b.begin_block(Some(label_id)).map_err(|e| e.to_string())?;

            let blk = cre.get(blk_id);
            for inst_id in cre.extra_get(blk.insts) {
              InstLow::low_inst(b, cre, ty_low, symbols, &mut fctx, inst_id)?;
            }

            InstLow::low_term(b, cre, ty_low, symbols, &fctx, &block_map, &blk.term)?;
          }

          b.end_function().map_err(|e| e.to_string())?;
        }

        SymbolKind::Variable { value: _, ism: _ } => {
          // Global variables if any
          let var_id = symbols[&sym_id];
          let ptr_ty = ty_low.pointer_type(b, cre, sym.ety, StorageClass::Uniform)?;
          let _ = b.variable(ptr_ty, Some(var_id), StorageClass::Uniform, None);
        }

        SymbolKind::Vmt { .. } => {}
      }
    }

    Ok(())
  }

  pub fn register_entry_points(
    b: &mut Builder,
    entry_points: &[EntryPointInfo],
  ) {
    for ep in entry_points {
      b.entry_point(ExecutionModel::GLCompute, ep.id, &ep.name, vec![]);
      b.execution_mode(ep.id, rspirv::spirv::ExecutionMode::LocalSize, vec![1, 1, 1]);
    }
  }
}
