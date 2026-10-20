/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::Itertools;
use qwc_diagnostic::{CollectOrErrors, Status::{self, Success}, Summary};
use qwc_ast as ast;
use qwc_hir::{self as hir, PushOkApi};

use crate::{Ctx, ItemLow, ctx, item_p::read_attrs, TypeLow};


pub fn low_krate(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Status<hir::ItemId> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();
  
  let ids = ctx.src.extra_get(rng)
    .filter_map(|id| ItemLow::low(ctx!(lscp -> ctx), sum, id).transpose())
    .collect_or_errors()?;

  let rng = ctx.cre.extra(&ids);

  let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;


  // Post
  hir::Item {
    name: it.name.map(|n| n.sid()),
    kind: hir::ItemKind::RootNS { rng },
    path: ctx.path,
    vis, attr
  }.push_ok(ctx.cre)
}

pub fn low_module(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Status<hir::ItemId> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();
  let name = it.name.unwrap().sid();
  let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });

  let ids = ctx.src.extra_get(rng)
    .filter_map(|id| ItemLow::low(ctx!(lscp -> ctx), sum, id).transpose())
    .collect_or_errors()?;

  let rng = ctx.cre.extra(&ids);

  let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;


  // Post
  hir::Item {
    name: Some(name),
    kind: hir::ItemKind::NameSpace { rng },
    path,
    vis, attr
  }.push_ok(ctx.cre)
}

pub fn low_generic(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng, args: ast::ThingRng, reqs: ast::ThingRng) -> Status<hir::ItemId> {
  let lscp = ctx.scp.get(&id.to_any()).unwrap();

  let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;

  let dummy_me = hir::Item {
    name: None,
    kind: hir::ItemKind::GenericNS { rng: hir::Rng::empty(ctx.cre.cid()), args: hir::Rng::empty(ctx.cre.cid()) },
    path: ctx.path,
    vis,
    attr,
  }.push(ctx.cre);

  ctx.cmap.cache_item.insert(id, Some(dummy_me));


  // Args
  let args = &ctx.src.extra_get(args)
    .filter_map(|id| {
      match ctx.src.get(id).kind {
        ast::ThingKind::Name(name) => Some(hir::Thing::Name(name.sid()).push(ctx.cre)),

        ast::ThingKind::NamedType(name, kind) => {
          TypeLow::low(ctx, sum, kind).map(|kind|
            hir::Thing::NamedType(name.sid(), kind).push(ctx.cre)
          ).ok()
        }
        _ => todo!()
      }
    }).collect_vec();
  let args = ctx.cre.extra(args);
  
  if let hir::ItemKind::GenericNS { args: args_me, .. } = &mut ctx.cre.get_mut(dummy_me).kind {
    *args_me = args;
  } else {
    unreachable!()
  }

  // Reqs
  let reqs = ctx.src.extra_get(reqs).map(|id| {
    let ast::ThingKind::NamedTypeList(name, traits) = ctx.src.get(id).kind else { unreachable!() };

    let traits = ctx.src.extra_get(traits).map(|id| TypeLow::low(ctx, sum, id)).collect_or_errors()?;
    
    Success(())
  }).collect_or_errors()?;


  // Rng
  let ids = ctx.src.extra_get(rng)
    .filter_map(|id| ItemLow::low(ctx!(lscp -> ctx), sum, id).transpose())
    .collect_or_errors()?;

  let rng = ctx.cre.extra(&ids);


  // Push
  *ctx.cre.get_mut(dummy_me) = hir::Item {
    name: None,
    kind: hir::ItemKind::GenericNS { rng, args },
    path: ctx.path,
    vis, attr
  };

  Status::Success(dummy_me)
}
