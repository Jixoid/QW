/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use inkwell::{GlobalVisibility, module::Linkage};
use qwc_mir::{AnyId, SymbId, Symbol, SymbolKind, SymbolStat, Type, id::NodeKind};

use crate::{CtxI, CtxM, BlokLow, cgen::SymbolVal, TypeLow};


pub struct SymbLow;

impl SymbLow {

  pub fn declare_symbols<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>) {
    for idx in 0..ictx.cre.symbols_len() {
      let symb_id = SymbId::from_any(AnyId::new(idx as u32, NodeKind::Symb));
      let it: &Symbol = ictx.cre.get(symb_id);
      let name = ictx.cre.sym_str(it.name);

      match it.kind {
        SymbolKind::Variable { ism } => {
          let ty = TypeLow::low_basic_cached(uctx, ictx, it.ety);
          let gv = ictx.mol.add_global(ty, None, name);
          gv.set_constant(!ism);

          match it.stat {
            SymbolStat::Private => {
              gv.set_linkage(Linkage::Private);
            }
            SymbolStat::Internal => {
              gv.set_linkage(Linkage::External);
              gv.set_visibility(GlobalVisibility::Hidden);
            }
            SymbolStat::Export => {
              gv.set_linkage(Linkage::External);
              gv.set_visibility(GlobalVisibility::Default);
            }
            SymbolStat::Import => {
              gv.set_linkage(Linkage::External);
            }
          }

          uctx.symbols.insert(symb_id, SymbolVal::Global(gv));
        }

        SymbolKind::Function { .. } => {
          let fn_any_ty = TypeLow::low(ictx, ictx.cre.get(it.ety));
          let fn_ty = fn_any_ty.into_function_type();
          let fv = ictx.mol.add_function(name, fn_ty, None);

          match it.stat {
            SymbolStat::Private => {
              fv.set_linkage(Linkage::Private);
            }
            SymbolStat::Internal => {
              fv.set_linkage(Linkage::External);
              fv.as_global_value().set_visibility(GlobalVisibility::Hidden);
            }
            SymbolStat::Export => {
              fv.set_linkage(Linkage::External);
              fv.as_global_value().set_visibility(GlobalVisibility::Default);
            }
            SymbolStat::Import => {
              fv.set_linkage(Linkage::External);
            }
          }

          uctx.symbols.insert(symb_id, SymbolVal::Function(fv));
        }

        SymbolKind::Vmt { .. } => {
          let ty = TypeLow::low_basic_cached(uctx, ictx, it.ety);
          let gv = ictx.mol.add_global(ty, None, name);
          gv.set_constant(true);
          gv.set_linkage(Linkage::Private);

          uctx.symbols.insert(symb_id, SymbolVal::Global(gv));
        }
      }
    }
  }


  pub fn define_symbols<'ctx>(uctx: &mut CtxM<'ctx>, ictx: &CtxI<'ctx, '_>) {
    for idx in 0..ictx.cre.symbols_len() {
      let symb_id = SymbId::from_any(AnyId::new(idx as u32, NodeKind::Symb));
      let it: &Symbol = ictx.cre.get(symb_id);

      match it.kind {
        SymbolKind::Variable{..} => {
          if it.stat != SymbolStat::Import {
            if let Some(SymbolVal::Global(gv)) = uctx.symbols.get(&symb_id).copied() {
              let ty = TypeLow::low_basic_cached(uctx, ictx, it.ety);
              gv.set_initializer(&ty.const_zero());
            }
          }
        }

        SymbolKind::Function{entry, blocks} => {
          if it.stat != SymbolStat::Import {
            if let Some(SymbolVal::Function(fv)) = uctx.symbols.get(&symb_id).copied() {
              let ty: &Type = ictx.cre.get(it.ety);
              BlokLow::low_fn(uctx, ictx, fv, ty, entry, blocks);
            }
          }
        }

        SymbolKind::Vmt { size, align, table } => {
          if it.stat != SymbolStat::Import {
            if let Some(SymbolVal::Global(gv)) = uctx.symbols.get(&symb_id).copied() {
              let struct_ty = gv.get_value_type().into_struct_type();
              let arch_int_ty = struct_ty.get_field_type_at_index(0).unwrap().into_int_type();
              let ptr_ty = struct_ty.get_field_type_at_index(2).unwrap().into_pointer_type();

              let mut field_vals: Vec<inkwell::values::BasicValueEnum> = vec![
                arch_int_ty.const_int(size, false).into(),
                arch_int_ty.const_int(align, false).into(),
                ptr_ty.const_null().into(),
              ];

              for method_symb_id in ictx.cre.extra_get(table) {
                if let Some(SymbolVal::Function(fv)) = uctx.symbols.get(&method_symb_id).copied() {
                  field_vals.push(fv.as_global_value().as_pointer_value().into());
                }
              }

              let struct_val = ictx.ctx.const_struct(&field_vals, false);
              gv.set_initializer(&struct_val);
            }
          }
        }
      }
    }
  }

}
