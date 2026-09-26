use inkwell::{GlobalVisibility, module::Linkage};
use qwc_mir::{AnyId, SymbId, Symbol, SymbolKind, SymbolStat, Type, id::NodeKind};

use crate::{blok_p::BlokLow, context::{CGenCtx, SymbolVal}, type_p::TypeLow};


pub struct SymbLow;

impl SymbLow {

  pub fn declare_symbols<'ctx>(cgen: &mut CGenCtx<'ctx, '_>) {
    for idx in 0..cgen.cre.symbols_len() {
      let symb_id = SymbId::from_any(AnyId::new(idx as u32, NodeKind::Symb));
      let it: &Symbol = cgen.cre.get(symb_id);
      let name = cgen.cre.sym_str(it.name);

      match it.kind {
        SymbolKind::Variable { ism } => {
          let ty = TypeLow::low_basic_cached(cgen.ctx, cgen.cre, it.ety, &mut cgen.type_cache);
          let gv = cgen.mol.add_global(ty, None, name);
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

          cgen.symbols.insert(symb_id, SymbolVal::Global(gv));
        }

        SymbolKind::Function { .. } => {
          let fn_any_ty = TypeLow::low(cgen.ctx, cgen.cre, cgen.cre.get(it.ety));
          let fn_ty = fn_any_ty.into_function_type();
          let fv = cgen.mol.add_function(name, fn_ty, None);

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

          cgen.symbols.insert(symb_id, SymbolVal::Function(fv));
        }
      }
    }
  }


  pub fn define_symbols<'ctx>(cgen: &mut CGenCtx<'ctx, '_>) {
    for idx in 0..cgen.cre.symbols_len() {
      let symb_id = SymbId::from_any(AnyId::new(idx as u32, NodeKind::Symb));
      let it: &Symbol = cgen.cre.get(symb_id);

      match it.kind {
        SymbolKind::Variable{..} => {
          if it.stat != SymbolStat::Import {
            if let Some(SymbolVal::Global(gv)) = cgen.symbols.get(&symb_id).copied() {
              let ty = TypeLow::low_basic_cached(cgen.ctx, cgen.cre, it.ety, &mut cgen.type_cache);
              gv.set_initializer(&ty.const_zero());
            }
          }
        }

        SymbolKind::Function{entry, blocks, stack} => {
          if it.stat != SymbolStat::Import {
            if let Some(SymbolVal::Function(fv)) = cgen.symbols.get(&symb_id).copied() {
              let ty: &Type = cgen.cre.get(it.ety);
              BlokLow::low(cgen, fv, ty, entry, blocks, stack);
            }
          }
        }
      }
    }
  }

}
