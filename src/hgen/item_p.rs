use std::assert_matches;

use crate::{ast::{self, AstKind, Rng}, diagnostic::Message, hgen::{GenContext, TypeGen}, hir::{self, Item}, lexer::{SrcLoc, SrcSId}};


pub struct ItemGen;

impl ItemGen {

  pub fn low_item(ctx: &mut GenContext, id: ast::ItemId) -> Result<Option<hir::ItemId>, Message> {
    let it = ctx.ast.get_item(id);

    let res = match &it.vari {
      ast::ItemVari::Module{name, ctn} => Some(Self::low_module(ctx, id, name, ctn)?),
      ast::ItemVari::Generic{params, reqs, ctn} => Self::low_generic(ctx, id, params, reqs, ctn)?,
      ast::ItemVari::Impl{type_ty, trait_ty, ctn} => Self::low_impl(ctx, type_ty, trait_ty, ctn)?,
      ast::ItemVari::Import(..) => None,
    };

    Ok(res)
  }

  pub fn low_decl(ctx: &mut GenContext, id: ast::DeclId) -> Result<Option<hir::ItemId>, Message> {
    let it = ctx.ast.get_decl(id);

    let id = match &it.vari {
      ast::DeclVari::Fun  { kind, blok } => Self::low_fun(ctx, kind, blok)?,
      ast::DeclVari::Init { kind, blok, ils } => Self::low_init(ctx, kind, blok, ils)?,
      ast::DeclVari::Fini { kind, blok } => Self::low_fini(ctx, kind, blok)?,
      ast::DeclVari::Var  { kind, init } => Self::low_var(ctx, kind, init)?,
      ast::DeclVari::Let  { kind, init } => Self::low_let(ctx, kind, init)?,
      
      ast::DeclVari::Using { kind } => { Self::low_using(ctx, kind)?; return Ok(None); }
    };

    Ok(Some(id))
  }


  fn low_module(ctx: &mut GenContext, id: ast::ItemId, _name: &String, ctn: &Rng /* DeclId | ItemId */) -> Result<hir::ItemId, Message> {
    ctx.global.enter(id);

    let mut items = vec![];

    for x in ctx.ast.get_extra(ctn) {
      match x.kind {
        AstKind::Decl => if let Some(item_id) = ItemGen::low_decl(ctx, ast::DeclId::new_from(*x))? { items.push(item_id); }
        AstKind::Item => if let Some(item_id) = ItemGen::low_item(ctx, ast::ItemId::new_from(*x))? { items.push(item_id); }

        _ => panic!("illegal type"),
      };
    }

    ctx.global.leave();

    let ctn_rng = ctx.hir.new_extra_from(&items);
    Ok(ctx.hir.new_item(Item::Module(ctn_rng)))
  }

  fn low_generic(ctx: &mut GenContext, id: ast::ItemId, params: &Rng /* NamedType */, reqs: &Rng /* NamedTypeList */, ctn: &Rng /* DeclId | ItemId */) -> Result<Option<hir::ItemId>, Message> {
    ctx.global.enter(id);

    for id in ctx.ast.get_extra(params) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));

      assert_matches!(it, ast::Thing::NamedType(..));

      if let ast::Thing::NamedType(_n, _t) = *it {}
    }

    for id in ctx.ast.get_extra(reqs) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));
      if let ast::Thing::NamedTypeList(span, traits_rng) = it {
        let param_name = span.str(ctx.far).to_string();
        let mut trait_tys = vec![];
        for tr_id in ctx.ast.get_extra(traits_rng) {
          if tr_id.kind == AstKind::Type {
            let tr_ty = TypeGen::low(ctx, ast::TypeId::new_from(*tr_id))?;
            trait_tys.push(tr_ty);
          }
        }
        ctx.type_bounds.insert(param_name, trait_tys);
      }
    }

    // Find any struct/type defined in this generic block to set as self_type
    let mut generic_self = None;
    for x in ctx.ast.get_extra(ctn) {
      if x.kind == AstKind::Decl {
        let decl = ctx.ast.get_decl(ast::DeclId::new_from(*x));
        if let ast::DeclVari::Using { kind } = &decl.vari {
          generic_self = Some(*kind);
          break;
        }
      }
    }

    let old_self = ctx.self_type;
    if generic_self.is_some() {
      ctx.self_type = generic_self;
    }

    let mut items = vec![];
    for x in ctx.ast.get_extra(ctn) {
      match x.kind {
        AstKind::Decl => if let Some(item_id) = ItemGen::low_decl(ctx, ast::DeclId::new_from(*x))? {
          items.push(item_id);
        }
        
        AstKind::Item => if let Some(item_id) = ItemGen::low_item(ctx, ast::ItemId::new_from(*x))? {
          items.push(item_id);
        }

        _ => panic!("illegal type"),
      };
    }

    ctx.self_type = old_self;
    ctx.global.leave();

    let ctn_rng = ctx.hir.new_extra_from(&items);
    Ok(Some(ctx.hir.new_item(Item::Module(ctn_rng))))
  }

  fn low_impl(ctx: &mut GenContext, type_ty: &ast::TypeId, trait_ty: &Option<ast::TypeId>, ctn: &Rng) -> Result<Option<hir::ItemId>, Message> {
    let old_self = ctx.self_type;
    ctx.self_type = Some(*type_ty);

    let _target_ty = TypeGen::low(ctx, *type_ty)?;

    let mut items = vec![];
    let mut provided_methods = std::collections::HashMap::new();

    for x in ctx.ast.get_extra(ctn) {
      match x.kind {
        AstKind::Decl => {
          let decl_id = ast::DeclId::new_from(*x);
          let decl = ctx.ast.get_decl(decl_id);
          provided_methods.insert(decl.name.sid(), decl_id);

          if let Some(item_id) = ItemGen::low_decl(ctx, decl_id)? {
            items.push(item_id);
          }
        }

        AstKind::Item => if let Some(item_id) = ItemGen::low_item(ctx, ast::ItemId::new_from(*x))? {
          items.push(item_id);
        }
        
        _ => panic!("illegal type in impl"),
      };
    }

    // Trait / Interface Conformance Checking
    if let Some(tr_ast_id) = trait_ty {
      let tr_hir_ty = TypeGen::low(ctx, *tr_ast_id)?;
      Self::check_impl_conformance(ctx, _target_ty, tr_hir_ty, *tr_ast_id, &provided_methods)?;
    }

    ctx.self_type = old_self;

    let ctn_rng = ctx.hir.new_extra_from(&items);
    Ok(Some(ctx.hir.new_item(Item::Module(ctn_rng))))
  }

  fn check_impl_conformance(
    ctx: &mut GenContext,
    target_hir_ty: hir::TypeId,
    trait_hir_ty: hir::TypeId,
    trait_ast_ty: ast::TypeId,
    provided_methods: &std::collections::HashMap<u32, ast::DeclId>,
  ) -> Result<(), Message> {
    let trait_ast = ctx.ast.get_type(trait_ast_ty);
    let req_funs_rng = match trait_ast {
      ast::Type::Trait { funs, .. } | ast::Type::Iface { funs, .. } => funs.clone(),
      ast::Type::Nick { pos, .. } => {
        if let Some(target_id) = ctx.global.lookup(ctx, pos.sid) {
          if target_id.kind == AstKind::Decl {
            let decl = ctx.ast.get_decl(ast::DeclId::new_from(target_id));
            if let ast::DeclVari::Using { kind } = &decl.vari {
              let real_trait = ctx.ast.get_type(*kind);
              match real_trait {
                ast::Type::Trait { funs, .. } | ast::Type::Iface { funs, .. } => funs.clone(),
                _ => return Ok(()),
              }
            } else { return Ok(()); }
          } else { return Ok(()); }
        } else { return Ok(()); }
      }
      _ => return Ok(()),
    };

    // Check for missing required methods
    for id in ctx.ast.get_extra(&req_funs_rng) {
      if id.kind == AstKind::Decl {
        let req_decl = ctx.ast.get_decl(ast::DeclId::new_from(*id));
        let req_name = req_decl.name.str(ctx.far);
        
        if let Some(&prov_decl_id) = provided_methods.get(&req_decl.name.sid) {
          // Check method signature match
          let prov_decl = ctx.ast.get_decl(prov_decl_id);
          let req_ty = match &req_decl.vari {
            ast::DeclVari::Fun { kind, .. } | ast::DeclVari::Init { kind, .. } | ast::DeclVari::Fini { kind, .. } => *kind,
            _ => continue,
          };
          let prov_ty = match &prov_decl.vari {
            ast::DeclVari::Fun { kind, .. } | ast::DeclVari::Init { kind, .. } | ast::DeclVari::Fini { kind, .. } => *kind,
            _ => continue,
          };

          let req_hir_ty = TypeGen::low(ctx, req_ty)?;
          let prov_hir_ty = TypeGen::low(ctx, prov_ty)?;

          if !Self::is_trait_match(ctx.hir, req_hir_ty, prov_hir_ty, trait_hir_ty, target_hir_ty) {
            return Err(Message::error(
              prov_decl.name,
              "method `{}` has incompatible signature for trait",
              vec![req_name.to_string()],
            ));
          }
        } else {
          return Err(Message::error(
            req_decl.name,
            "missing required method `{}` for trait implementation",
            vec![req_name.to_string()],
          ));
        }
      }
    }

    Ok(())
  }

  fn is_trait_match(
    hir: &hir::Crate,
    req_ty: hir::TypeId,
    prov_ty: hir::TypeId,
    trait_ty: hir::TypeId,
    target_ty: hir::TypeId,
  ) -> bool {
    if (req_ty == trait_ty || hir.get_type(req_ty) == hir.get_type(trait_ty)) && (prov_ty == target_ty || hir.get_type(prov_ty) == hir.get_type(target_ty)) {
      return true;
    }
    if req_ty == prov_ty {
      return true;
    }

    let t_req = hir.get_type(req_ty);
    let t_prov = hir.get_type(prov_ty);

    match (t_req, t_prov) {
      (hir::Type::Fun { args: a1, ret: r1 }, hir::Type::Fun { args: a2, ret: r2 }) => {
        let args1 = hir.get_extra(a1);
        let args2 = hir.get_extra(a2);
        if args1.len() != args2.len() { return false; }
        for (x1, x2) in args1.iter().zip(args2.iter()) {
          let th1 = hir.get_thing(hir::ThingId::new_from(*x1));
          let th2 = hir.get_thing(hir::ThingId::new_from(*x2));
          match (th1, th2) {
            (hir::Thing::NamedType(_, sub1), hir::Thing::NamedType(_, sub2)) => {
              if !Self::is_trait_match(hir, *sub1, *sub2, trait_ty, target_ty) {
                return false;
              }
            }
            _ => return false,
          }
        }

        match (r1, r2) {
          (Some(sub1), Some(sub2)) => Self::is_trait_match(hir, *sub1, *sub2, trait_ty, target_ty),
          (None, None) => true,
          _ => false,
        }
      }

      (hir::Type::Ref(sub1, acc1), hir::Type::Ref(sub2, acc2)) => {
        acc1 == acc2 && Self::is_trait_match(hir, *sub1, *sub2, trait_ty, target_ty)
      }
      (hir::Type::Ptr(sub1, acc1), hir::Type::Ptr(sub2, acc2)) => {
        acc1 == acc2 && Self::is_trait_match(hir, *sub1, *sub2, trait_ty, target_ty)
      }

      _ => hir.is_same_type(req_ty, prov_ty),
    }
  }


  fn bind_function_params(ctx: &mut GenContext, ast_fn_ty_id: ast::TypeId) -> Result<(), Message> {
    ctx.local.enter();

    // Bind `self` if inside a struct/type method
    if let Some(self_ty) = ctx.self_type {
      let h_self_ty = TypeGen::low(ctx, self_ty)?;
      let self_ref_ty = ctx.hir.new_type(hir::Type::Ref(h_self_ty, hir::AccessKind::MUT));
      
      for (sid, s) in ctx.ast.str_pool.iter().enumerate() {
        if s == "self" {
          ctx.local.bind(sid as u32, self_ref_ty);
          break;
        }
      }
    }

    // Bind parameters
    let fn_ast = ctx.ast.get_type(ast_fn_ty_id);
    let args_rng = match fn_ast {
      ast::Type::Fun { args, .. } | ast::Type::Init { args, .. } | ast::Type::Fini { args, .. } => args.clone(),
      _ => 0..0,
    };

    for id in ctx.ast.get_extra(&args_rng) {
      if id.kind == ast::AstKind::Thing {
        let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));
        if let ast::Thing::NamedType(span, param_ty) = it {
          let h_param_ty = TypeGen::low(ctx, *param_ty)?;
          ctx.local.bind(span.sid(), h_param_ty);
        }
      }
    }

    Ok(())
  }


  fn low_fun(ctx: &mut GenContext, kind: &ast::TypeId, blok: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeGen::low(ctx, *kind)?;

    let blok = if let Some(b) = blok {
      Self::bind_function_params(ctx, *kind)?;
      let h_blok = crate::hgen::ExprGen::low(ctx, *b)?;
      ctx.local.leave();
      Some(h_blok)
    } else {
      None
    };

    let this = Item::Fun { kind: hir_kind, blok };
    Ok(ctx.hir.new_item(this))
  }

  fn low_init(ctx: &mut GenContext, kind: &ast::TypeId, blok: &Option<ast::ExprId>, _ils: &Option<ast::Rng>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeGen::low(ctx, *kind)?;

    let blok = if let Some(b) = blok {
      Self::bind_function_params(ctx, *kind)?;
      let h_blok = crate::hgen::ExprGen::low(ctx, *b)?;
      ctx.local.leave();
      Some(h_blok)
    } else {
      None
    };

    let this = Item::Fun { kind: hir_kind, blok };
    Ok(ctx.hir.new_item(this))
  }

  fn low_fini(ctx: &mut GenContext, kind: &ast::TypeId, blok: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeGen::low(ctx, *kind)?;

    let blok = if let Some(b) = blok {
      Self::bind_function_params(ctx, *kind)?;
      let h_blok = crate::hgen::ExprGen::low(ctx, *b)?;
      ctx.local.leave();
      Some(h_blok)
    } else {
      None
    };

    let this = Item::Fun { kind: hir_kind, blok };
    Ok(ctx.hir.new_item(this))
  }

  fn low_var(ctx: &mut GenContext, kind: &ast::TypeId, init: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let ty = TypeGen::low(ctx, *kind)?;

    let init = if let Some(i) = init {
      Some(crate::hgen::ExprGen::low(ctx, *i)?)
    } else {
      None
    };

    let this = Item::Var { kind: ty, init };
    Ok(ctx.hir.new_item(this))
  }

  fn low_let(ctx: &mut GenContext, kind: &ast::TypeId, init: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let ty = TypeGen::low(ctx, *kind)?;
    
    let init = if let Some(i) = init {
      Some(crate::hgen::ExprGen::low(ctx, *i)?)
    } else {
      None
    };

    let this = Item::Let { kind: ty, init };
    Ok(ctx.hir.new_item(this))
  }

  fn low_using(ctx: &mut GenContext, kind: &ast::TypeId) -> Result<(), Message> {
    let old_self = ctx.self_type;
    ctx.self_type = Some(*kind);
    let _ = TypeGen::low(ctx, *kind)?;
    ctx.self_type = old_self;
    
    Ok(())
  }

}
