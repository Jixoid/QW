use itertools::Itertools;
use qwc_diagnostic::Message;
use qwc_ast as ast;
use qwc_hir::{self as hir, PushOkApi};

use crate::{Ctx, ctx, ItemLow, item_p::read_attrs};


pub fn low_krate(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();
  
  let ids = &ctx.src.extra_get(rng)
    .filter_map(|id| {
      ItemLow::low(ctx!(lscp -> ctx), id)
        .map_err(|err| ctx.sum.add(err))
        .ok().flatten()
    }).collect_vec();

  let rng = ctx.cre.extra(ids);

  let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


  // Post
  hir::Item {
    kind: hir::ItemKind::RootNS { rng },
    vis, attr
  }.push_ok(ctx.cre)
}

pub fn low_module(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();

  let ids = &ctx.src.extra_get(rng)
    .filter_map(|id| {
      ItemLow::low(ctx!(lscp -> ctx), id)
        .map_err(|err| ctx.sum.add(err))
        .ok().flatten()
    }).collect_vec();

  let rng = ctx.cre.extra(ids);

  let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


  // Post
  hir::Item {
    kind: hir::ItemKind::NameSpace {
      name: it.name.unwrap().sid(),
      rng,
    },
    vis, attr
  }.push_ok(ctx.cre)
}

pub fn low_generic(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();

  let ids = &ctx.src.extra_get(rng)
    .filter_map(|id| {
      ItemLow::low(ctx!(lscp -> ctx), id)
        .map_err(|err| ctx.sum.add(err))
        .ok().flatten()
    }).collect_vec();

  let rng = ctx.cre.extra(ids);

  let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;

  
  // Post
  hir::Item {
    kind: hir::ItemKind::GenericNS { rng },
    vis, attr
  }.push_ok(ctx.cre)
}
