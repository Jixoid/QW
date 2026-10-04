use itertools::Itertools;
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Attribute, Ident};
use qwc_hir::{self as hir, PushOkApi};
use qwc_string_interner::StrInterner;
use rustc_hash::FxHashMap;

use crate::{Ctx, ExprLow, TypeLow, ctx};


pub struct ItemLow;

impl ItemLow {

  pub fn low(ctx: &mut Ctx, id: ast::ItemId) -> Result<Option<hir::ItemId>, Message> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);

    let it = match it.kind {
      ast::ItemKind::Krate(rng) => Some(Self::low_krate(ctx, id, it, rng)?),

      ast::ItemKind::Module(rng) | ast::ItemKind::ModuleFile(rng, ..) => Some(Self::low_module(ctx, id, it, rng)?),
      
      ast::ItemKind::Generic{ctn: rng, ..} => Some(Self::low_generic(ctx, id, it, rng)?),

      
      ast::ItemKind::Using(kind) | ast::ItemKind::ItemTy(kind) => Some(Self::low_using(ctx, id, it, kind)?),
      


      // Symbols
      ast::ItemKind::Fun{kind, blok} => Some(Self::low_fun(ctx, id, it, kind, blok)?),

      ast::ItemKind::Let{kind, value, ism} => Some(Self::low_let(ctx, id, it, kind, value, ism)?),
      
      // Unexpected
      ast::ItemKind::ModuleUnloaded => panic!("ast object that should not be present"),
      
      ast::ItemKind::Impl{type_ty, trait_ty, ctn} => Some(Self::low_impl(ctx, id, it, type_ty, trait_ty, ctn)?),
      ast::ItemKind::Import(..) => None,
    };

    ctx.cmap.cache_item.insert(id, it);

    Ok(it)
  }


  fn low_krate(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();
    
    let ids = &ctx.src.extra_get(rng)
      .filter_map(|id| {
        Self::low(ctx!(lscp -> ctx), id)
          .map_err(|err| ctx.sum.add(err))
          .ok().flatten()
      }).collect_vec();

    let rng = ctx.cre.extra(ids);

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::RootNS { rng },
      vis: convert_vis(it.vis),
      svis
    }.push_ok(ctx.cre)
  }

  fn low_module(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let ids = &ctx.src.extra_get(rng)
      .filter_map(|id| {
        Self::low(ctx!(lscp -> ctx), id)
          .map_err(|err| ctx.sum.add(err))
          .ok().flatten()
      }).collect_vec();

    let rng = ctx.cre.extra(ids);

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::NameSpace {
        name: it.name.unwrap().sid(),
        rng,
      },
      vis: convert_vis(it.vis),
      svis
    }.push_ok(ctx.cre)
  }

  fn low_generic(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let ids = &ctx.src.extra_get(rng)
      .filter_map(|id| {
        Self::low(ctx!(lscp -> ctx), id)
          .map_err(|err| ctx.sum.add(err))
          .ok().flatten()
      }).collect_vec();

    let rng = ctx.cre.extra(ids);

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;

    
    // Post
    hir::Item {
      kind: hir::ItemKind::GenericNS { rng },
      vis: convert_vis(it.vis),
      svis
    }.push_ok(ctx.cre)
  }


  fn low_using(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Result<hir::ItemId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Using {
        name: it.name.unwrap().sid(),
        kind,
      },
      vis: convert_vis(it.vis),
      svis,
    }.push_ok(ctx.cre)
  }


  fn low_fun(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    Self::low_fun_helper(ctx, it.name.unwrap(), kind, expr, it.pos, it.vis, ctx.src.get_attached(id))
  }

  fn low_fun_helper(ctx: &mut Ctx, name: Ident, kind: ast::TypeId, expr: Option<ast::ExprId>, pos: Span, vis: ast::Visibility, attached: Option<&Vec<Attribute>>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind, args: ast_args, ..} = (ctx.src.get(kind) as &ast::Type).kind else { unreachable!() };
    let hir::TypeKind::Fun{args: hir_args, .. } = (ctx.cre.get(hir_kind) as &hir::Type).kind else { unreachable!() };
      
    let mut hir_args_iter = ctx.cre.extra_get(hir_args);

    if self_kind.is_some() {
      let self_arg_id = hir_args_iter.next().unwrap();
      let self_sid = ctx.sin.sid_self();
      loc.insert(self_sid, self_arg_id, false, pos);
    }

    for (thing_id, arg_id) in ctx.src.extra_get(ast_args).zip(hir_args_iter) {
      let thing: &ast::Thing = ctx.src.get(thing_id);
      if let ast::Thing::NamedType(name, _) = *thing {
        if let Some(old_id) = loc.lookup(&name.sid()) {
          let old_span = loc.get_local(old_id).span;
          return Err(Message::error(DUPLICATE_IDENTIFIER, Label::new_pos(name)).add(Label::new(old_span, FIRST_DEFINITION_HERE)));
        }

        loc.insert(name.sid(), arg_id, false, name.into());
      }
    }

    let expr = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;

    let svis = read_attrs(ctx.sin, attached)?;

    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Function {
        name: name.sid(),
        expr,
        kind: hir_kind,
      },
      vis: convert_vis(vis),
      svis
    };

    Ok(ctx.cre.push(this))
  }

  fn type_ident_name<'a>(src: &'a ast::Krate, far: &'a qwc_arena::Files, id: ast::TypeId) -> Option<&'a str> {
    let it = src.get(id);
    match it.kind {
      ast::TypeKind::Nick(ident) => Some(ident.str(far)),
      ast::TypeKind::Path(rng) => {
        let last = src.extra_get(rng).last()?;
        let it = src.get(last);
        if let ast::TypeKind::Nick(ident) = it.kind {
          Some(ident.str(far))
        } else {
          None
        }
      }
      _ => None,
    }
  }

  fn validate_and_order_iface_impl(ctx: &mut Ctx, iface_hir_ty: hir::TypeId, struct_hir_ty: hir::TypeId, trait_ast_ty: Option<ast::TypeId>, implemented_methods: Vec<(Ident, hir::ItemId, Span)>, impl_span: Span) -> Result<hir::ItemRng, Message> {
    let iface = ctx.cre.get(iface_hir_ty);
    
    let hir::TypeKind::Iface(iface_methods) = iface.kind else { unreachable!() };

    let iface_name = trait_ast_ty
      .and_then(|id| Self::type_ident_name(ctx.src, ctx.far, id))
      .unwrap_or("iface")
      .to_string();

    let mut expected_methods = vec![];
    for mid in ctx.cre.extra_get(iface_methods) {
      let hir::Thing::NamedType(name_sid, fun_ty) = *ctx.cre.get(mid) else { unreachable!() };
      expected_methods.push((name_sid, fun_ty));
    }

    let mut impl_map: FxHashMap<qwc_string_interner::Sid, (Ident, hir::ItemId, Span)> = FxHashMap::default();
    for (ident, method_id, pos) in implemented_methods {
      let sid = ident.sid();

      if let Some((.., first_pos)) = impl_map.get(&sid) {
        return Err(Message::error(DUPLICATE_IDENTIFIER, Label::new_pos(ident))
          .add(Label::new(*first_pos, FIRST_DEFINITION_HERE)));
      }

      impl_map.insert(sid, (ident, method_id, pos));
    }

    // Extra method check
    for (sid, (ident, _, pos)) in &impl_map {
      if !expected_methods.iter().any(|(exp_sid, _)| exp_sid == sid) {
        let method_name = ident.str(ctx.far);

        return Err(Message::error(
          METHOD_NOT_A_MEMBER_OF_IFACE.args(&[method_name, &iface_name]),
          Label::new(*pos, NOT_A_MEMBER_OF_IFACE.args(&[&iface_name])),
        ));
      }
    }

    // Missing method check
    let mut missing = vec![];

    for (exp_sid, _) in &expected_methods {
      if !impl_map.contains_key(exp_sid) {
        missing.push(format!("`{}`", ctx.sin.str(*exp_sid)));
      }
    }

    if !missing.is_empty() {
      let missing_str = missing.join(", ");
      return Err(Message::error(
        NOT_ALL_IFACE_ITEMS_IMPLEMENTED.args(&[&missing_str]),
        Label::new(impl_span, MISSING_IN_IMPLEMENTATION.args(&[&missing_str])),
      ));
    }


    // Signature matching
    for (exp_sid, exp_fun_ty) in &expected_methods {
      let (ident, method_id, pos) = impl_map.get(exp_sid).unwrap();
      let hir::ItemKind::Function { kind: act_fun_ty, .. } = ctx.cre.get(*method_id).kind else { unreachable!() };
      let exp_fun = ctx.cre.get(*exp_fun_ty);
      let act_fun = ctx.cre.get(act_fun_ty);
      let (hir::TypeKind::Fun { args: exp_args, ret: exp_ret }, hir::TypeKind::Fun { args: act_args, ret: act_ret }) = (exp_fun.kind, act_fun.kind) else { unreachable!() };

      let exp_args_list: Vec<hir::TypeId> = ctx.cre.extra_get(exp_args).collect();
      let act_args_list: Vec<hir::TypeId> = ctx.cre.extra_get(act_args).collect();

      if exp_args_list.len() != act_args_list.len() {
        return Err(Message::error(
          FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED.args(&[&exp_args_list.len().to_string(), &act_args_list.len().to_string()]),
          Label::new_pos(*pos),
        ));
      }

      if !exp_args_list.is_empty() {
        let exp_first = ctx.cre.get(exp_args_list[0]);
        let act_first = ctx.cre.get(act_args_list[0]);

        if let hir::TypeKind::Ref(_exp_sub, exp_ism) = exp_first.kind {
          if let hir::TypeKind::Ref(act_sub, act_ism) = act_first.kind {
            if exp_ism != act_ism || act_sub != struct_hir_ty {
              let method_name = ident.str(ctx.far);
              return Err(Message::error(
                INCOMPATIBLE_IFACE_METHOD_TYPE.args(&[method_name, &iface_name]),
                Label::new(*pos, EXPECTED_X.args(&[&format!("&{}self", if exp_ism { "mut " } else { "" })])),
              ));
            }
          } else {
            let method_name = ident.str(ctx.far);
            return Err(Message::error(
              INCOMPATIBLE_IFACE_METHOD_TYPE.args(&[method_name, &iface_name]),
              Label::new(*pos, EXPECTED_X.args(&[&format!("&{}self", if exp_ism { "mut " } else { "" })])),
            ));
          }
        } else if exp_args_list[0] != act_args_list[0] {
          let method_name = ident.str(ctx.far);
          return Err(Message::error(
            INCOMPATIBLE_IFACE_METHOD_TYPE.args(&[method_name, &iface_name]),
            Label::new(*pos, EXPECTED_X.args(&[&ctx.type_name(exp_args_list[0])])),
          ));
        }

        for idx in 1..exp_args_list.len() {
          if exp_args_list[idx] != act_args_list[idx] {
            let method_name = ident.str(ctx.far);
            return Err(Message::error(
              INCOMPATIBLE_IFACE_METHOD_TYPE.args(&[method_name, &iface_name]),
              Label::new(*pos, EXPECTED_X.args(&[&ctx.type_name(exp_args_list[idx])])),
            ));
          }
        }
      }

      if exp_ret != act_ret {
        let method_name = ident.str(ctx.far);
        return Err(Message::error(
          INCOMPATIBLE_IFACE_METHOD_TYPE.args(&[method_name, &iface_name]),
          Label::new(*pos, EXPECTED_X.args(&[&ctx.type_name(exp_ret)])),
        ));
      }
    }


    // Order methods according to iface declaration order
    let mut ordered_ids = Vec::with_capacity(expected_methods.len());
    for (exp_sid, _) in &expected_methods {
      let (_, method_id, _) = impl_map.get(exp_sid).unwrap();
      ordered_ids.push(*method_id);
    }

    Ok(ctx.cre.extra(&ordered_ids))
  }

  fn low_impl(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, type_ty: ast::TypeId, trait_ty: Option<ast::TypeId>, ctn: ast::FieldRng) -> Result<hir::ItemId, Message> {
    let struct_hir_ty = TypeLow::low(ctx, type_ty)?;
    let iface_hir_ty = trait_ty.map(|ty| TypeLow::low(ctx, ty)).transpose()?;

    let mut method_ids = vec![];
    for method_item_id in ctx.src.extra_get(ctn) {
      let method_item = ctx.src.get(method_item_id);
      let ast::FieldKind::Fun { kind, blok } = method_item.kind else { panic!() };
      let name = method_item.name.unwrap();

      ctx.cmap.self_ty.push(struct_hir_ty);

      let method_id = Self::low_fun_helper(
        ctx,
        name,
        kind,
        blok,
        method_item.pos,
        method_item.vis,
        ctx.src.get_attached(method_item_id),
      )?;

      ctx.cmap.self_ty.pop();

      method_ids.push((name, method_id, method_item.pos));
    }

    let methods = if let Some(iface_ty) = iface_hir_ty {
      Self::validate_and_order_iface_impl(
        ctx,
        iface_ty,
        struct_hir_ty,
        trait_ty,
        method_ids,
        it.pos,
      )?
    } else {
      let ids: Vec<_> = method_ids.into_iter().map(|(_, id, _)| id).collect();
      ctx.cre.extra(&ids)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Impl {
        struct_ty: struct_hir_ty,
        iface_ty: iface_hir_ty,
        methods,
      },
      vis: convert_vis(it.vis),
      svis,
    };

    Ok(ctx.cre.push(this))
  }

  fn low_let(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: Option<ast::TypeId>, expr: ast::ExprId, ism: bool) -> Result<hir::ItemId, Message> {
    let kind = kind.unwrap();
    let kind = TypeLow::low(ctx, kind)?;

    let expr = ExprLow::low(ctx, expr)?;

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;
    

    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Variable {
        name: it.name.unwrap().sid(),
        kind,
        expr,
        ism,
      },
      vis: convert_vis(it.vis),
      svis,
    };

    Ok(ctx.cre.push(this))
  }

}


fn read_attrs(sin: &StrInterner, attrs: Option<&Vec<Attribute>>) -> Result<Option<hir::SymVis>, Message> {
  let mut ivis: Option<(ast::Ident, hir::SymVis)> = None;

  if let Some(attrs) = attrs {
    for attr in attrs {
      let key = attr.ident;
      
      match () {
        _ if key.sid() == sin.sid_import() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Import))
        },

        _ if key.sid() == sin.sid_export() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Export))
        },
      
        _ => return Err(Message::error(UNKNOWN_ATTRIBUTE, Label::new_pos(key)))
      }
    }
  }

  Ok(ivis.map(|val| val.1))
}


fn convert_vis(vis: ast::Visibility) -> hir::ItemVis {
  match vis {
    ast::Visibility::Inherited => hir::ItemVis::Private,
    ast::Visibility::Public  => hir::ItemVis::Public,
    ast::Visibility::Private => hir::ItemVis::Private,
    _ => panic!()
  }
}
