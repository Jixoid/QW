/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


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
    name: it.name.map(|n| n.sid()),
    kind: hir::ItemKind::RootNS { rng },
    path: ctx.path,
    vis, attr
  }.push_ok(ctx.cre)
}

pub fn low_module(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();
  let name = it.name.unwrap().sid();
  let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });

  let ids = &ctx.src.extra_get(rng)
    .filter_map(|id| {
      ItemLow::low(ctx!(lscp, path -> ctx), id)
        .map_err(|err| ctx.sum.add(err))
        .ok().flatten()
    }).collect_vec();

  let rng = ctx.cre.extra(ids);

  let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


  // Post
  hir::Item {
    name: Some(name),
    kind: hir::ItemKind::NameSpace { rng },
    path,
    vis, attr
  }.push_ok(ctx.cre)
}

pub fn low_generic(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng, args: ast::ThingRng) -> Result<hir::ItemId, Message> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();

  let rng = &ctx.src.extra_get(rng)
    .filter_map(|id| {
      ItemLow::low(ctx!(lscp -> ctx), id)
        .map_err(|err| ctx.sum.add(err))
        .ok().flatten()
    }).collect_vec();
  let rng = ctx.cre.extra(rng);
  
  
  let args = &ctx.src.extra_get(args)
    .map(|id| {
      match ctx.src.get(id).kind {
        ast::ThingKind::Name(name) => hir::Thing::NamedType(name.sid(), ctx.prims.ty_type).push(ctx.cre),
        //ast::ThingKind::NamedType(name, kind)

        _ => todo!()
      }
    }).collect_vec();
  let args = ctx.cre.extra(args);

  let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;

  
  // Post
  hir::Item {
    name: None,
    kind: hir::ItemKind::GenericNS { rng, args },
    path: ctx.path,
    vis, attr
  }.push_ok(ctx.cre)
}
