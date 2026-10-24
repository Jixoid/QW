/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::assert_matches;

use qwc_diagnostic::{Status::{self, Success}, Summary};
use qwc_hir::{self as hir, ItemAttr};
use qwc_mir::{self as mir, PushOkApi};
use qwc_mangling::{Mangler, ManglerQW};

use crate::{BlokLow, Ctx, TypeLow, FunBuilder, Layouter, expr_p::ExprLow};


pub struct SymbLow;

impl SymbLow {

  pub fn low(ctx: &mut Ctx, sum: &mut Summary, id: hir::ItemId) -> Status<Option<mir::SymbId>> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Success(id) }

    if id.cid() != ctx.src.cid() {
      let it = ctx.get(id);
      let symb = match it.kind {
        hir::ItemKind::Function { kind, .. } => {
          let sym = ctx.get_symbol(id).unwrap().to_string();
          let ety = TypeLow::low(ctx, sum, kind)?;
          let symb = mir::Symbol {
            name: ctx.cre.sym(&sym),
            stat: mir::SymbolStat::Import,
            ety,
            kind: mir::SymbolKind::Function {
              entry: mir::BlokId::from_any(mir::AnyId::new(0, mir::id::NodeKind::Blok)),
              blocks: mir::BlokRng::empty(),
            },
          }.push(ctx.cre);
          Some(symb)
        }
        hir::ItemKind::Variable { kind, ism, .. } => {
          let sym = ctx.get_symbol(id).unwrap().to_string();
          let ety = TypeLow::low(ctx, sum, kind)?;
          let symb = mir::Symbol {
            name: ctx.cre.sym(&sym),
            stat: mir::SymbolStat::Import,
            ety,
            kind: mir::SymbolKind::Variable {
              value: mir::Value::Const(mir::Const::Unit),
              ism,
            },
          }.push(ctx.cre);
          Some(symb)
        }
        _ => None,
      };
      ctx.cmap.cache_item.insert(id, symb);
      return Success(symb);
    }

    let it = ctx.src.get(id);

    use hir::ItemKind::*;

    let it = match it.kind {
      RootNS {rng} => {Self::low_root(ctx, sum, rng)?; None},
      NameSpace {rng} => {Self::low_namespace(ctx, sum, rng)?; None}

      Using {..} => None,

      Variable {kind, expr, ism} => Some(Self::low_variable(ctx, sum, it, kind, expr, ism)?),
      Function {kind, expr} => Some(Self::low_function(ctx, sum, it, kind, expr)?),

      Impl {type_ty, trait_ty, methods} => {Self::low_impl(ctx, sum, it, type_ty, trait_ty, methods)?; None}

      GenericNS{..} => None,

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_item.insert(id, it);

    Success(it)
  }


  fn low_root(ctx: &mut Ctx, sum: &mut Summary, rng: hir::ItemRng) -> Status<()> {
    for id in ctx.src.extra_get(rng) {
      Self::low(ctx, sum, id)?;
    }

    Success(())
  }
  
  fn low_namespace(ctx: &mut Ctx, sum: &mut Summary, rng: hir::ItemRng) -> Status<()> {
    for id in ctx.src.extra_get(rng) {
      Self::low(ctx, sum, id)?;
    }

    Success(())
  }


  fn low_variable(ctx: &mut Ctx, sum: &mut Summary, it: &hir::Item, kind: hir::TypeId, expr: hir::ExprId, ism: bool) -> Status<mir::SymbId> {
    let sym = ManglerQW::new(ctx.src, ctx.deps, ctx.sin, it.path);

    let ety = TypeLow::low(ctx, sum, kind)?;

    let value = ExprLow::low(ctx, sum, &mut FunBuilder::new(), expr)?.unwrap();

    assert_matches!(value, mir::Value::Const(..) | mir::Value::GlobalRef(..));


    // Post
    mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: convert_vis(it),
      ety,
      kind: mir::SymbolKind::Variable{ value, ism }
    }.push_ok(ctx.cre)
  }

  fn low_function(ctx: &mut Ctx, sum: &mut Summary, it: &hir::Item, kind: hir::TypeId, expr: hir::ExprId) -> Status<mir::SymbId> {
    let sym = if it.attr.contains(ItemAttr::Entry) {
      String::from("qw_entry")
    } else {
      ManglerQW::new(ctx.src, ctx.deps, ctx.sin, it.path)
    };
    
    let ety = TypeLow::low(ctx, sum, kind)?;

    let is_ret_unit = {
      let ty: &mir::Type = ctx.cre.get(ety);
      match ty.kind {
        mir::TypeKind::Fun { ret, .. } => {
          let ret_ty: &mir::Type = ctx.cre.get(ret);
          matches!(ret_ty.kind, mir::TypeKind::Unit)
        }
        _ => false,
      }
    };

    let param_tys: Vec<mir::TypeId> = if let mir::TypeKind::Fun{args, ..} = (ctx.cre.get(ety) as &mir::Type).kind {
      ctx.cre.extra_get(args).collect()
    } else {
      unreachable!()
    };

    let (entry, blocks) = BlokLow::low_fn(ctx, sum, expr, is_ret_unit, &param_tys)?;


    // Post
    mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: convert_vis(it),
      ety,
      kind: mir::SymbolKind::Function{ entry, blocks }
    }.push_ok(ctx.cre)
  }


  fn low_method_sym(ctx: &mut Ctx, sum: &mut Summary, it: &hir::Item, kind: hir::TypeId, expr: hir::ExprId) -> Status<mir::SymbId> {
    let sym = ManglerQW::new(ctx.src, ctx.deps, ctx.sin, it.path);
    
    let ety = TypeLow::low(ctx, sum, kind)?;

    let is_ret_unit = {
      let mir::TypeKind::Fun{ret, ..} = ctx.cre.get(ety).kind else { unreachable!() };
      
      let ret_ty = ctx.cre.get(ret);
      matches!(ret_ty.kind, mir::TypeKind::Unit)
    };
    
    let param_tys: Vec<mir::TypeId> = if let mir::TypeKind::Fun{args, ..} = ctx.cre.get(ety).kind {
      ctx.cre.extra_get(args).collect()
    } else {
      unreachable!()
    };

    let (entry, blocks) = BlokLow::low_fn(ctx, sum, expr, is_ret_unit, &param_tys)?;


    // Post
    mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: convert_vis(it),
      ety,
      kind: mir::SymbolKind::Function{ entry, blocks }
    }.push_ok(ctx.cre)
  }

  fn low_impl(ctx: &mut Ctx, sum: &mut Summary, it: &hir::Item, struct_ty: hir::TypeId, trait_ty: Option<hir::TypeId>, methods: hir::ItemRng) -> Status<()> {
    let is_iface = trait_ty.map(|id| if let hir::TypeKind::Iface(..) = ctx.get(id).kind { true } else { false }).unwrap_or(false);


    let mut method_symbs = vec![];

    for id in ctx.src.extra_get(methods) {
      if let Some(&mir_id) = ctx.cmap.cache_item.get(&id) {
        if let Some(symb_id) = mir_id {
          method_symbs.push(symb_id);
        }
        continue;
      }
      let it = ctx.src.get(id);
      let mir_id = if let hir::ItemKind::Function { kind, expr } = it.kind {
        let symb_id = Self::low_method_sym(ctx, sum, it, kind, expr)?;
        method_symbs.push(symb_id);
        Some(symb_id)
      } else {
        None
      };
      ctx.cmap.cache_item.insert(id, mir_id);
    }

    if is_iface {
      let struct_mir_ty = TypeLow::low(ctx, sum, struct_ty)?;
      let struct_type = ctx.cre.get(struct_mir_ty);
      let size = struct_type.layout.size().unwrap_or(0);
      let align = struct_type.layout.align().unwrap_or(1) as u64;

      let vmt_sym_name = ManglerQW::new(ctx.src, ctx.deps, ctx.sin, it.path);
      let table = ctx.cre.extra(&method_symbs);

      let arch_int_ty = ctx.tin.ty_arch_int();
      let ptr_ty = ctx.tin.ty_ptr();

      let mut vmt_fields = vec![arch_int_ty, arch_int_ty, ptr_ty];
      for _ in 0..method_symbs.len() {
        vmt_fields.push(ptr_ty);
      }

      let rng = ctx.cre.extra(&vmt_fields);
      let struct_kind = mir::TypeKind::Struct(rng);
      let ety = mir::Type {
        kind: struct_kind,
        layout: Layouter::layout(&struct_kind, ctx.tin.layinfo, ctx.cre),
      }.push(ctx.cre);

      let vmt_id = mir::Symbol {
        name: ctx.cre.sym(&vmt_sym_name),
        kind: mir::SymbolKind::Vmt { size, align, table },
        stat: mir::SymbolStat::Private,
        ety,
      }.push(ctx.cre);
      ctx.cmap.cache_vmt.insert((struct_ty, trait_ty.unwrap()), vmt_id);
    }

    Success(())
  }

  pub fn get_or_low_vmt(ctx: &mut Ctx, sum: &mut Summary, struct_ty: hir::TypeId, iface_ty: hir::TypeId) -> Status<mir::SymbId> {
    if let Some(&symb_id) = ctx.cmap.cache_vmt.get(&(struct_ty, iface_ty)) {
      return Success(symb_id);
    }

    let root = ctx.src.root().unwrap();
    let hir::ItemKind::RootNS { rng } = ctx.src.get(root).kind else { unreachable!() };
    
    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);
      if let hir::ItemKind::Impl { type_ty: s, trait_ty: Some(i), methods } = it.kind {
        if s == struct_ty && i == iface_ty {
          Self::low_impl(ctx, sum, it, s, Some(i), methods)?;
          return Success(*ctx.cmap.cache_vmt.get(&(struct_ty, iface_ty)).expect("vmt must be cached"));
        }
      }
    }

    panic!("VMT not found for struct {:?} and iface {:?}", struct_ty, iface_ty);
  }

}


fn convert_vis(it: &hir::Item) -> mir::SymbolStat {
  match it.vis {
    hir::ItemVis::Private => mir::SymbolStat::Private,

    hir::ItemVis::Public(svis) => match svis {
      hir::SymVis::Import => mir::SymbolStat::Import,
      hir::SymVis::Export => mir::SymbolStat::Export,
      hir::SymVis::Internal => mir::SymbolStat::Internal,
    }
  }
}
