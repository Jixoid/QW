/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Ident};
use qwc_hir::{self as hir, PushOkApi};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;

use crate::{Ctx, TypeLow, TypeMatch, ItemLow, item_p::read_attrs};


pub fn low_impl(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, type_ty: ast::TypeId, trait_ty: Option<ast::TypeId>, ctn: ast::FieldRng) -> Result<hir::ItemId, Message> {
  let type_ty = TypeLow::low(ctx, type_ty)?;
  let trait_ty = trait_ty.map(|id| TypeLow::low(ctx, id)).transpose()?;

  let method_ids = {
    let mut vec = vec![];
    
    for id in ctx.src.extra_get(ctn) {
      let it = ctx.src.get(id);
      
      let ast::FieldKind::Fun { kind, blok } = it.kind else { panic!() };
      
      ctx.cmap.self_ty.push(type_ty);
      let mid = ItemLow::low_fun_field(ctx, id, it, kind, blok)?;
      ctx.cmap.self_ty.pop();
      
      vec.push((it.name.unwrap(), mid, kind, it.pos));
    }

    vec
  };
  
  
  let methods = if let Some(trait_ty) = trait_ty {
    low_impl_validate(ctx, type_ty, trait_ty, it.pos, method_ids)?
  } else {
    let ids: Vec<_> = method_ids.into_iter().map(|(_, id, ..)| id).collect();
    ctx.cre.extra(&ids)
  };

  let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


  // Post
  hir::Item {
    kind: hir::ItemKind::Impl {
      type_ty,
      trait_ty,
      methods,
    },
    vis, attr
  }.push_ok(ctx.cre)
}

fn low_impl_validate(ctx: &mut Ctx, _type_ty: hir::TypeId, trait_ty: hir::TypeId, impl_span: Span, implemented_methods: Vec<(Ident, hir::ItemId, ast::TypeId, Span)>) -> Result<hir::ItemRng, Message> {
  let (hir::TypeKind::Iface(trait_methods) | hir::TypeKind::Trait(trait_methods)) = ctx.cre.get(trait_ty).kind else { unreachable!() };


  // Expected Methods
  let expected_methods = ctx.cre.extra_get(trait_methods)
    .map(|id| {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { unreachable!() };
      (name, kind)
    }).collect_vec().into_boxed_slice();

  
  // Implementation Map
  let impl_map = {
    let mut map: FxHashMap<Sid, (Ident, hir::ItemId, Span)> = FxHashMap::default();
    
    for (ident, method_id, _, pos) in &implemented_methods {
      
      if let Some((.., first_pos)) = map.get(&ident.sid()) {
        return Err(Message::error(DUPLICATE_IDENTIFIER, Label::new_pos(*ident))
          .add(Label::new(*first_pos, FIRST_DEFINITION_HERE)));
      }
      
      map.insert(ident.sid(), (*ident, *method_id, *pos));
    }

    map
  };


  // Extra method check
  for (sid, (ident, _, pos)) in &impl_map {
    if !expected_methods.iter().any(|(exp_sid, _)| exp_sid == sid) {
      return Err(Message::error(METHOD_NOT_A_MEMBER_OF_IFACE.args(&[ident.str(ctx.far)]), Label::new(*pos, NOT_A_MEMBER_OF_IFACE)));
    }
  }


  // Missing method check
  {
    let missing = expected_methods.iter().map(|(sid, _)| sid)
      .filter_map(|sid| {
        (!impl_map.contains_key(sid)).then(|| format!("`{}`", ctx.sin.str(*sid)))
      })
      .collect_vec().into_boxed_slice();
    
    if !missing.is_empty() {
      let missing_str = missing.join(", ");
      return Err(Message::error(NOT_ALL_IFACE_ITEMS_IMPLEMENTED.args(&[&missing_str]), Label::new(impl_span, MISSING_IN_IMPLEMENTATION.args(&[&missing_str]))));
    };
  };


  // Signature matching
  for ((exp_sid, exp_fun), (_, _, fun2_ast, _)) in izip!(&expected_methods, &implemented_methods) {
    let (_, method_id, _) = impl_map.get(exp_sid).unwrap();
    
    let hir::ItemKind::Function{kind: act_fun, ..} = ctx.cre.get(*method_id).kind else { unreachable!() };

    TypeMatch::match_fun(ctx, *exp_fun, act_fun, *fun2_ast)?;
  }


  // Order methods according to iface declaration order
  let mut ordered_ids = Vec::with_capacity(expected_methods.len());
  for (exp_sid, _) in &expected_methods {
    let (_, method_id, _) = impl_map.get(exp_sid).unwrap();
    ordered_ids.push(*method_id);
  }

  Ok(ctx.cre.extra(&ordered_ids))
}
