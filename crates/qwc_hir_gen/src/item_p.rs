use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Attribute, Ident};
use qwc_hir::{self as hir, ItemAttr, PushOkApi};
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;

use crate::{Ctx, ExprLow, TypeLow, ctx, TypeMatch};


pub struct ItemLow;

impl ItemLow {

  pub fn low(ctx: &mut Ctx, id: ast::ItemId) -> Result<Option<hir::ItemId>, Message> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);

    let it = match it.kind {
      // Scope
      ast::ItemKind::Krate(rng) => Some(Self::low_krate(ctx, id, it, rng)?),

      ast::ItemKind::Module(rng) | ast::ItemKind::ModuleFile(rng, ..) => Some(Self::low_module(ctx, id, it, rng)?),
      
      ast::ItemKind::Generic{ctn: rng, ..} => Some(Self::low_generic(ctx, id, it, rng)?),


      // Symbols
      ast::ItemKind::Fun{kind, blok} => Some(Self::low_fun(ctx, id, it, kind, blok)?),
      
      ast::ItemKind::Task{kind, blok} => Some(Self::low_task(ctx, id, it, kind, blok)?),
      
      ast::ItemKind::Let{kind, value, ism} => Some(Self::low_let(ctx, id, it, kind, value, ism)?),
      
      ast::ItemKind::Impl{type_ty, trait_ty, ctn} => Some(Self::low_impl(ctx, id, it, type_ty, trait_ty, ctn)?),


      // Using
      ast::ItemKind::Using(kind) | ast::ItemKind::ItemTy(kind) => Some(Self::low_using(ctx, id, it, kind)?),


      // Unexpected
      ast::ItemKind::ModuleUnloaded => panic!("ast object that should not be present"),
      
      // Impl
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

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::RootNS { rng },
      vis, attr
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

  fn low_generic(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let ids = &ctx.src.extra_get(rng)
      .filter_map(|id| {
        Self::low(ctx!(lscp -> ctx), id)
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


  fn low_using(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Result<hir::ItemId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Using {
        name: it.name.unwrap().sid(),
        kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }


  fn low_let(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: Option<ast::TypeId>, expr: ast::ExprId, ism: bool) -> Result<hir::ItemId, Message> {
    let kind = kind.unwrap();
    let kind = TypeLow::low(ctx, kind)?;

    let expr = ExprLow::low(ctx, expr)?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;
    

    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Variable {
        name: it.name.unwrap().sid(),
        kind,
        expr,
        ism,
      },
      vis, attr
    };

    Ok(ctx.cre.push(this))
  }


  fn low_fun(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ret} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      let ast::Thing::NamedType(name_pos, ..) = *ctx.src.get(id_pos) else { panic!() };

      loc.insert(name, kind, false, name_pos);
    }

    let expr_hir = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;
    
    let expr_pos = ctx.src.get(expr.unwrap()).pos;
    let expr_ty = ctx.cre.get(expr_hir).ety;

    let ret = if let hir::TypeKind::Trait(..) = ctx.cre.get(ret).kind && let hir::TypeKind::TraitFrom{..} = ctx.cre.get(expr_ty).kind {
      let fun_sign = ctx.cre.get_mut(hir_kind);

      let hir::TypeKind::Fun {ret: sign_ret, ..} = &mut fun_sign.kind else { panic!() };

      *sign_ret = expr_ty;
      expr_ty
    } else { ret };

    TypeMatch::matches_pos(ctx, ret, expr_ty, expr_pos)?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Function {
        name: it.name.unwrap().sid(),
        expr: expr_hir,
        kind: hir_kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }

  fn low_task(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ..} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      let ast::Thing::NamedType(name_pos, ..) = *ctx.src.get(id_pos) else { panic!() };

      loc.insert(name, kind, false, name_pos);
    }

    let expr = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Task {
        name: it.name.unwrap().sid(),
        expr,
        kind: hir_kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }

  fn low_fun_field(ctx: &mut Ctx, id: ast::FieldId, it: &ast::Field, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ..} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      let ast::Thing::NamedType(name_pos, ..) = *ctx.src.get(id_pos) else { panic!() };

      loc.insert(name, kind, false, name_pos);
    }

    let expr = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Function {
        name: it.name.unwrap().sid(),
        expr,
        kind: hir_kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }


  fn low_impl(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, type_ty: ast::TypeId, trait_ty: Option<ast::TypeId>, ctn: ast::FieldRng) -> Result<hir::ItemId, Message> {
    let type_ty = TypeLow::low(ctx, type_ty)?;
    let trait_ty = trait_ty.map(|id| TypeLow::low(ctx, id)).transpose()?;

    let method_ids = {
      let mut vec = vec![];
      
      for id in ctx.src.extra_get(ctn) {
        let it = ctx.src.get(id);
        
        let ast::FieldKind::Fun { kind, blok } = it.kind else { panic!() };
        
        ctx.cmap.self_ty.push(type_ty);
        let mid = Self::low_fun_field(ctx, id, it, kind, blok)?;
        ctx.cmap.self_ty.pop();
        
        vec.push((it.name.unwrap(), mid, kind, it.pos));
      }

      vec
    };
    
    
    let methods = if let Some(trait_ty) = trait_ty {
      Self::low_impl_validate(ctx, type_ty, trait_ty, it.pos, method_ids)?
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

}


fn read_attrs(ctx: &Ctx, vis: ast::Visibility, attrs: Option<&Vec<Attribute>>) -> Result<(hir::ItemVis, ItemAttr), Message> {
  let mut ivis: Option<(ast::Ident, hir::SymVis)> = None;
  let mut attr = ItemAttr::empty();


  if let Some(attrs) = attrs {
    for key in attrs {
      let key = key.ident;
      
      match () {
        _ if key.sid() == ctx.sin.sid_import() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Import))
        },

        _ if key.sid() == ctx.sin.sid_export() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Export))
        },

        _ if key.sid() == ctx.sin.sid_entry() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          if attr.contains(ItemAttr::Entry) {
            return Err(Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //.add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          if vis != ast::Visibility::Public {
            return Err(Message::error(ENTRY_FUNCTION_MUST_BE_PUBLIC, Label::new_pos(key))
              
            )
          }
          ivis = Some((key, hir::SymVis::Export));
          attr |= ItemAttr::Entry;
        }
      
        _ => return Err(Message::error(UNKNOWN_ATTRIBUTE.args(&[key.str(ctx.far)]), Label::new_pos(key)))
      }
    }
  }



  let vis = match vis {
    ast::Visibility::Inherited => hir::ItemVis::Private,
    ast::Visibility::Public  => hir::ItemVis::Public(ivis.unwrap().1),
    ast::Visibility::Private => hir::ItemVis::Private,
    _ => panic!()
  };

  Ok((vis, attr))
}
