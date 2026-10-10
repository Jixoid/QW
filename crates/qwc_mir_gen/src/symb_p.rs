/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Message;
use qwc_hir::{self as hir, ItemAttr};
use qwc_mir as mir;
use qwc_mangling::{Mangler, ManglerQW};

use crate::{BlokLow, Ctx, MayFail, TypeLow, FunBuilder, Layouter, expr_p::ExprLow};


pub struct SymbLow;

impl SymbLow {

  pub fn low(ctx: &mut Ctx, id: hir::ItemId) -> Result<Option<mir::SymbId>, Message> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);

    use hir::ItemKind::*;

    let it = match it.kind {
      RootNS {rng} => {Self::low_root(ctx, rng)?; None},
      NameSpace {rng} => {Self::low_namespace(ctx, rng)?; None}

      Using {..} => None,

      Variable {kind, expr, ism} => Some(Self::low_variable(ctx, it, kind, expr, ism)?),
      Function {kind, expr} => Some(Self::low_function(ctx, it, kind, expr)?),

      Impl {type_ty, trait_ty, methods} => {Self::low_impl(ctx, it, type_ty, trait_ty, methods)?; None}

      GenericNS{..} => None,

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_item.insert(id, it);

    Ok(it)
  }


  fn low_root(ctx: &mut Ctx, rng: hir::ItemRng) -> MayFail<Message> {
    for id in ctx.src.extra_get(rng) {
      Self::low(ctx, id)?;
    }

    Ok(())
  }
  
  fn low_namespace(ctx: &mut Ctx, rng: hir::ItemRng) -> MayFail<Message> {
    for id in ctx.src.extra_get(rng) {
      Self::low(ctx, id)?;
    }

    Ok(())
  }


  fn low_variable(ctx: &mut Ctx, it: &hir::Item, kind: hir::TypeId, expr: hir::ExprId, ism: bool) -> Result<mir::SymbId, Message> {
    let sym = ManglerQW::new(ctx.src, ctx.sin, it.path);

    let ety = TypeLow::low(ctx, kind)?;

    let _value = ExprLow::low(ctx, &mut FunBuilder::new(), expr)?;


    // Post
    let this = mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: convert_vis(it),
      ety,
      kind: mir::SymbolKind::Variable{ ism }
    };

    Ok(ctx.cre.push(this))
  }

  fn low_function(ctx: &mut Ctx, it: &hir::Item, kind: hir::TypeId, expr: hir::ExprId) -> Result<mir::SymbId, Message> {
    let sym = if it.attr.contains(ItemAttr::Entry) {
      String::from("qw_entry")
    } else {
      ManglerQW::new(ctx.src, ctx.sin, it.path)
    };
    
    let ety = TypeLow::low(ctx, kind)?;

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

    let (entry, blocks) = BlokLow::low_fn(ctx, expr, is_ret_unit, &param_tys)?;


    // Post
    let this = mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: convert_vis(it),
      ety,
      kind: mir::SymbolKind::Function{ entry, blocks }
    };

    Ok(ctx.cre.push(this))
  }


  fn low_method_sym(ctx: &mut Ctx, it: &hir::Item, kind: hir::TypeId, expr: hir::ExprId) -> Result<mir::SymbId, Message> {
    let sym = ManglerQW::new(ctx.src, ctx.sin, it.path);
    
    let ety = TypeLow::low(ctx, kind)?;

    let is_ret_unit = {
      let mir::TypeKind::Fun{ret, ..} = ctx.cre.get(ety).kind else { panic!() };
      
      let ret_ty = ctx.cre.get(ret);
      matches!(ret_ty.kind, mir::TypeKind::Unit)
    };
    
    let param_tys: Vec<mir::TypeId> = if let mir::TypeKind::Fun{args, ..} = ctx.cre.get(ety).kind {
      ctx.cre.extra_get(args).collect()
    } else {
      unreachable!()
    };

    let (entry, blocks) = BlokLow::low_fn(ctx, expr, is_ret_unit, &param_tys)?;


    // Post
    let this = mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: convert_vis(it),
      ety,
      kind: mir::SymbolKind::Function{ entry, blocks }
    };

    Ok(ctx.cre.push(this))
  }

  fn low_impl(ctx: &mut Ctx, it: &hir::Item, struct_ty: hir::TypeId, trait_ty: Option<hir::TypeId>, methods: hir::ItemRng) -> MayFail<Message> {
    let is_iface = trait_ty.map(|id| if let hir::TypeKind::Iface(..) = ctx.src.get(id).kind { true } else { false }).unwrap_or(false);


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
        let symb_id = Self::low_method_sym(ctx, it, kind, expr)?;
        method_symbs.push(symb_id);
        Some(symb_id)
      } else {
        None
      };
      ctx.cmap.cache_item.insert(id, mir_id);
    }

    if is_iface {
      let struct_mir_ty = TypeLow::low(ctx, struct_ty)?;
      let struct_type = ctx.cre.get(struct_mir_ty);
      let size = struct_type.layout.size().unwrap_or(0);
      let align = struct_type.layout.align().unwrap_or(1) as u64;

      let vmt_sym_name = ManglerQW::new(ctx.src, ctx.sin, it.path);
      let table = ctx.cre.extra(&method_symbs);

      let arch_int_ty = ctx.tin.ty_arch_int();
      let ptr_ty = ctx.tin.ty_ptr();

      let mut vmt_fields = vec![arch_int_ty, arch_int_ty, ptr_ty];
      for _ in 0..method_symbs.len() {
        vmt_fields.push(ptr_ty);
      }

      let rng = ctx.cre.extra(&vmt_fields);
      let struct_kind = mir::TypeKind::Struct(rng);
      let ety = ctx.cre.push(mir::Type {
        kind: struct_kind,
        layout: Layouter::layout(&struct_kind, ctx.tin.layinfo, ctx.cre, qwc_hir::LayoutBy::QW),
      });

      let this = mir::Symbol {
        name: ctx.cre.sym(&vmt_sym_name),
        kind: mir::SymbolKind::Vmt { size, align, table },
        stat: mir::SymbolStat::Private,
        ety,
      };
      let vmt_id = ctx.cre.push(this);
      ctx.cmap.cache_vmt.insert((struct_ty, trait_ty.unwrap()), vmt_id);
    }

    Ok(())
  }

  pub fn get_or_low_vmt(ctx: &mut Ctx, struct_ty: hir::TypeId, iface_ty: hir::TypeId) -> Result<mir::SymbId, Message> {
    if let Some(&symb_id) = ctx.cmap.cache_vmt.get(&(struct_ty, iface_ty)) {
      return Ok(symb_id);
    }

    let root = ctx.src.root().unwrap();
    let hir::ItemKind::RootNS { rng } = ctx.src.get(root).kind else { panic!() };
    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);
      if let hir::ItemKind::Impl { type_ty: s, trait_ty: Some(i), methods } = it.kind {
        if s == struct_ty && i == iface_ty {
          Self::low_impl(ctx, it, s, Some(i), methods)?;
          return Ok(*ctx.cmap.cache_vmt.get(&(struct_ty, iface_ty)).expect("vmt must be cached"));
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
