use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Attribute, Ident};
use qwc_hir::{self as hir};
use qwc_string_interner::StrInterner;

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

    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        if let Some(hir_id) = Self::low(ctx!(lscp -> ctx), id)? {
          ctn.push(hir_id);
        }
        Self::collect_struct_impls(ctx!(lscp -> ctx), id, &mut ctn)?;
      }

      ctx.cre.extra(&ctn)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::RootNS { rng },
      vis: convert_vis(it.vis),
      svis
    };
    
    Ok(ctx.cre.push(this))
  }

  fn low_module(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        if let Some(hir_id) = Self::low(ctx!(lscp -> ctx), id)? {
          ctn.push(hir_id);
        }
        Self::collect_struct_impls(ctx!(lscp -> ctx), id, &mut ctn)?;
      }

      ctx.cre.extra(&ctn)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::NameSpace {
        name: it.name.unwrap().sid(),
        rng,
      },
      vis: convert_vis(it.vis),
      svis
    };
    
    Ok(ctx.cre.push(this))
  }

  fn low_generic(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        if let Some(hir_id) = Self::low(ctx!(lscp -> ctx), id)? {
          ctn.push(hir_id);
        }
        Self::collect_struct_impls(ctx!(lscp -> ctx), id, &mut ctn)?;
      }

      ctx.cre.extra(&ctn)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;

    
    // Post
    let this = hir::Item {
      kind: hir::ItemKind::GenericNS { rng },
      vis: convert_vis(it.vis),
      svis
    };

    Ok(ctx.cre.push(this))
  }


  fn low_using(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Result<hir::ItemId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Using {
        name: it.name.unwrap().sid(),
        kind,
      },
      vis: convert_vis(it.vis),
      svis,
    };

    Ok(ctx.cre.push(this))
  }


  fn low_fun(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    Self::low_fun_helper(ctx, it.name.unwrap(), kind, expr, it.pos, it.vis, ctx.src.get_attached(id), None)
  }

  fn low_fun_helper(ctx: &mut Ctx, name: Ident, kind: ast::TypeId, expr: Option<ast::ExprId>, pos: Span, vis: ast::Visibility, attached: Option<&Vec<Attribute>>, self_ty: Option<hir::TypeId>) -> Result<hir::ItemId, Message> {
    ctx.cmap.self_ty = self_ty;
    let hir_kind = TypeLow::low(ctx, kind)?;
    ctx.cmap.self_ty = None;

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

  fn collect_struct_impls(ctx: &mut Ctx, id: ast::ItemId, ctn: &mut Vec<hir::ItemId>) -> Result<(), Message> {
    let it = ctx.src.get(id);
    let ast_ty_id = match it.kind {
      ast::ItemKind::Using(kind) | ast::ItemKind::ItemTy(kind) => kind,
      _ => return Ok(()),
    };

    let ast_ty = ctx.src.get(ast_ty_id);
    let ast::TypeKind::Struct(fields) = ast_ty.kind else { return Ok(()) };

    let struct_hir_ty = *ctx.cmap.cache_type.get(&ast_ty_id).expect("struct type must be cached");

    for field_id in ctx.src.extra_get(fields) {
      let field = ctx.src.get(field_id);
      
      let ast::FieldKind::ImplIn { trait_ty, ctn: method_rng } = field.kind else { continue };
      
      let iface_hir_ty = Some(TypeLow::low(ctx, trait_ty)?);
      let mut method_ids = vec![];

      for method_field_id in ctx.src.extra_get(method_rng) {
        let method_field = ctx.src.get(method_field_id);
        let ast::FieldKind::Fun { kind, blok } = method_field.kind else { continue };
        let method_id = Self::low_fun_helper(
          ctx,
          method_field.name.unwrap(),
          kind,
          blok,
          method_field.pos,
          method_field.vis,
          ctx.src.get_attached(method_field_id),
          Some(struct_hir_ty),
        )?;
        method_ids.push(method_id);
      }

      let methods = ctx.cre.extra(&method_ids);


      // Post
      let impl_item = hir::Item {
        kind: hir::ItemKind::Impl {
          struct_ty: struct_hir_ty,
          iface_ty: iface_hir_ty,
          methods,
        },
        vis: convert_vis(field.vis),
        svis: None,
      };
      ctn.push(ctx.cre.push(impl_item));
    }
    Ok(())
  }

  fn low_impl(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, type_ty: ast::TypeId, trait_ty: Option<ast::TypeId>, ctn: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let struct_hir_ty = TypeLow::low(ctx, type_ty)?;
    let iface_hir_ty = trait_ty.map(|ty| TypeLow::low(ctx, ty)).transpose()?;

    let mut method_ids = vec![];
    for method_item_id in ctx.src.extra_get(ctn) {
      let method_item = ctx.src.get(method_item_id);
      let ast::ItemKind::Fun { kind, blok } = method_item.kind else { panic!() };

      let method_id = Self::low_fun_helper(
        ctx,
        method_item.name.unwrap(),
        kind,
        blok,
        method_item.pos,
        method_item.vis,
        ctx.src.get_attached(method_item_id),
        Some(struct_hir_ty),
      )?;
      method_ids.push(method_id);
    }
    let methods = ctx.cre.extra(&method_ids);

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
