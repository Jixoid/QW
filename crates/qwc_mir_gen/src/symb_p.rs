use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;
use qwc_mangling::{Mangler, ManglerQW};
use qwc_string_interner::Sid;

use crate::{BlokLow, Ctx, MayFail, TypeLow, BlockBuilder, ctx, expr_p::ExprLow};


pub struct SymbLow;

impl SymbLow {

  pub fn low(ctx: &mut Ctx, id: hir::ItemId) -> Result<Option<mir::SymbId>, Message> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Ok(id) }

    let it: &hir::Item = ctx.src.get(id);

    let it = match it.kind {
      hir::ItemKind::RootNS{rng} => {Self::low_root(ctx, rng)?; None},
      hir::ItemKind::NameSpace{rng, name} => {Self::low_namespace(ctx, rng, name)?; None}

      hir::ItemKind::Variable{kind, name, expr, ism} => Some(Self::low_variable(ctx, it, name, kind, expr, ism)?),
      hir::ItemKind::Function{kind, name, expr} => Some(Self::low_function(ctx, name, kind, expr)?),

      kind @_ => todo!("{kind:#?}")
    };

    ctx.cmap.cache_item.insert(id, it);

    Ok(it)
  }


  fn low_root(ctx: &mut Ctx, rng: hir::Rng) -> MayFail<Message> {
    let mgr = &mut vec![];
    
    for id in ctx.src.extra_get(rng) {
      let id = hir::ItemId::new_from(id);

      Self::low(ctx!(mgr -> ctx), id)?;
    }

    Ok(())
  }
  
  fn low_namespace(ctx: &mut Ctx, rng: hir::Rng, name: Sid) -> MayFail<Message> {
    ctx.mgr.push(name);
    
    for id in ctx.src.extra_get(rng) {
      let id = hir::ItemId::new_from(id);

      Self::low(ctx, id)?;
    }

    ctx.mgr.pop();

    Ok(())
  }


  fn low_variable(ctx: &mut Ctx, _it: &hir::Item, name: Sid, kind: hir::TypeId, expr: hir::ExprId, ism: bool) -> Result<mir::SymbId, Message> {
    let sym = ManglerQW::new(ctx.sin, ctx.mgr, name);
    
    let ety = TypeLow::low(ctx, kind)?;

    let _value = ExprLow::low(ctx, &mut BlockBuilder::new(), expr)?;


    // Post
    let this = mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: mir::SymbolStat::Export,
      ety,
      kind: mir::SymbolKind::Variable{ ism }
    };

    Ok(ctx.cre.push(this))
  }

  fn low_function(ctx: &mut Ctx, name: Sid, kind: hir::TypeId, expr: hir::ExprId) -> Result<mir::SymbId, Message> {
    let sym = ManglerQW::new(ctx.sin, ctx.mgr, name);
    
    let ety = TypeLow::low(ctx, kind)?;

    let blok = BlokLow::low(ctx, expr)?;


    // Post
    let this = mir::Symbol {
      name: ctx.cre.sym(&sym),
      stat: mir::SymbolStat::Export,
      ety,
      kind: mir::SymbolKind::Function{ blok }
    };

    Ok(ctx.cre.push(this))
  }
  
}
