/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir as mir;

use crate::{adapter::{low_type, low_val}, cgen::Ctx, InstLow};


pub struct SymbLow;

impl SymbLow {
  pub fn low(ctx: &mut Ctx, id: mir::SymbId) -> Result<(), String> {
    let it = ctx.src.get(id);
    let name = ctx.src.sym_str(it.name);

    match it.kind {
      mir::SymbolKind::Function { entry, blocks: _ } => {
        Self::low_function(ctx, name, it.ety, entry)?;
      }

      mir::SymbolKind::Variable { .. } => {
        // Module global variable if needed, or skip
      }

      mir::SymbolKind::Vmt { .. } => {
        return Err("VMT / dynamic dispatch is not supported in CIRCT".to_string());
      }
    }

    Ok(())
  }


  fn low_function(ctx: &mut Ctx, name: &str, ety: mir::TypeId, entry: mir::BlokId) -> Result<(), String> {
    let fun_ty = ctx.src.get(ety);
    let mir::TypeKind::Fun { args, ret } = fun_ty.kind else {
      return Err(format!("expected function type for symbol {}, got {:?}", name, fun_ty.kind));
    };

    let mut ports = Vec::new();
    for (idx, arg_id) in ctx.src.extra_get(args).into_iter().enumerate() {
      let ty_str = low_type(ctx.src, arg_id)?;
      ports.push(format!("in %arg{} : {}", idx, ty_str));
    }

    let is_ret_unit = matches!(ctx.src.get(ret).kind, mir::TypeKind::Unit);
    if !is_ret_unit {
      let ret_ty_str = low_type(ctx.src, ret)?;
      ports.push(format!("out out : {}", ret_ty_str));
    }

    let ports_str = ports.join(", ");
    writeln!(ctx.adap.stream(), "hw.module @{}({}) {{", name, ports_str).unwrap();

    let entry_block = ctx.src.get(entry);
    for inst_id in ctx.src.extra_get(entry_block.insts) {
      InstLow::low(ctx, inst_id)?;
    }

    match entry_block.term {
      mir::Terminator::Return(Some(val)) => {
        let val_str = low_val(&val)?;
        let ret_ty_str = low_type(ctx.src, ret)?;
        writeln!(ctx.adap.stream(), "  hw.output {} : {}", val_str, ret_ty_str).unwrap();
      }
      mir::Terminator::Return(None) => {
        writeln!(ctx.adap.stream(), "  hw.output").unwrap();
      }
      _ => return Err("only simple Return terminator is currently supported in CIRCT functions".to_string()),
    }

    writeln!(ctx.adap.stream(), "}}\n").unwrap();
    Ok(())
  }
}
