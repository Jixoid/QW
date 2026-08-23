use std::assert_matches;

use crate::{ast::{self, AstKind, Rng}, diagnostic::Message, hgen::{GenContext, TypeGen}, hir::{self, Item}};


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
      
      ast::DeclVari::Using{..} => { return Ok(None); },
    };

    Ok(Some(id))
  }


  fn low_module(ctx: &mut GenContext, id: ast::ItemId, _name: &String, ctn: &Rng /* DeclId | ItemId */) -> Result<hir::ItemId, Message> {
    ctx.enter_scope(id.to_any());

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

    ctx.leave_scope();

    let ctn_rng = ctx.hir.new_extra(items);
    Ok(ctx.hir.new_item(Item::Module(ctn_rng)))
  }

  fn low_generic(ctx: &mut GenContext, id: ast::ItemId, params: &Rng /* NamedType */, _reqs: &Rng /* NamedTypeList */, ctn: &Rng /* DeclId | ItemId */) -> Result<Option<hir::ItemId>, Message> {
    ctx.enter_scope(id.to_any());

    for id in ctx.ast.get_extra(params) {
      let it = ctx.ast.get_thing(ast::ThingId::new_from(*id));

      assert_matches!(it, ast::Thing::NamedType(..));

      if let ast::Thing::NamedType(_n, _t) = *it {

      }
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

    ctx.leave_scope();

    let ctn_rng = ctx.hir.new_extra(items);
    Ok(Some(ctx.hir.new_item(Item::Module(ctn_rng))))
  }

  fn low_impl(ctx: &mut GenContext, type_ty: &ast::TypeId, _trait_ty: &Option<ast::TypeId>, ctn: &Rng) -> Result<Option<hir::ItemId>, Message> {
    let old_self = ctx.self_type;
    ctx.self_type = Some(*type_ty);

    let _target_ty = TypeGen::low(ctx, *type_ty)?;

    let mut items = vec![];
    for x in ctx.ast.get_extra(ctn) {
      match x.kind {
        AstKind::Decl => if let Some(item_id) = ItemGen::low_decl(ctx, ast::DeclId::new_from(*x))? {
          items.push(item_id);
        }

        AstKind::Item => if let Some(item_id) = ItemGen::low_item(ctx, ast::ItemId::new_from(*x))? {
          items.push(item_id);
        }
        
        _ => panic!("illegal type in impl"),
      };
    }

    ctx.self_type = old_self;

    let ctn_rng = ctx.hir.new_extra(items);
    Ok(Some(ctx.hir.new_item(Item::Module(ctn_rng))))
  }


  fn low_fun(ctx: &mut GenContext, kind: &ast::TypeId, _blok: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let kind = TypeGen::low(ctx, *kind)?;

    let this = Item::Fun(kind);

    Ok(ctx.hir.new_item(this))
  }

  fn low_init(ctx: &mut GenContext, kind: &ast::TypeId, _blok: &Option<ast::ExprId>, _ils: &Option<ast::Rng>) -> Result<hir::ItemId, Message> {
    let kind = TypeGen::low(ctx, *kind)?;

    let this = Item::Fun(kind);

    Ok(ctx.hir.new_item(this))
  }

  fn low_fini(ctx: &mut GenContext, kind: &ast::TypeId, _blok: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let kind = TypeGen::low(ctx, *kind)?;

    let this = Item::Fun(kind);

    Ok(ctx.hir.new_item(this))
  }

  fn low_var(ctx: &mut GenContext, kind: &ast::TypeId, _init: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let ty = TypeGen::low(ctx, *kind)?;

    let this = Item::Var(ty);
  
    Ok(ctx.hir.new_item(this))
  }

  fn low_let(ctx: &mut GenContext, kind: &ast::TypeId, _init: &Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let ty = TypeGen::low(ctx, *kind)?;

    let this = Item::Let(ty);

    Ok(ctx.hir.new_item(this))
  }

}
