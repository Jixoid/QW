use qwc_diagnostic::{Label, Message, msg::*};
use qwc_ast::{self as ast, Ident};
use qwc_hir as hir;
use qwc_resolve::{self as resolve, Resolver};

use crate::{Ctx, ExprLow, ctx};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }
    
    let it: &ast::Type = ctx.src.get(id);
  
    let it = match it.kind {
      ast::TypeKind::Nick(ident) => Self::low_nick(ctx, ident)?,
      ast::TypeKind::Path(rng)     => Self::low_path(ctx, rng)?,
      
      ast::TypeKind::Struct(rng) => Self::low_struct(ctx, rng)?,
      ast::TypeKind::Tuple(rng)  => Self::low_tuple(ctx, rng)?,
      
      ast::TypeKind::Array(kind, ext) => Self::low_array(ctx, kind, ext)?,
      
      ast::TypeKind::Fun{args, ret, ..} => Self::low_fun(ctx, args, ret)?,
      
      ast::TypeKind::Slice(kind) => Self::low_slice(ctx, kind)?,
      
      ast::TypeKind::Ref(id, _) => {let id = Self::low(ctx, id)?; ctx.cre.ty_ref(id)},
      
      ast::TypeKind::Unit => ctx.cre.ty_unit(),
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_type.insert(id, it);

    Ok(it)
  }


  fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::TypeId, Message> {
    let (kind, lscp, _span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp).lookup(ident)?.get_k();
    Self::low_resolved(ctx, kind, lscp)
  }

  fn low_path(ctx: &mut Ctx, rng: ast::Rng) -> Result<hir::TypeId, Message> {
    let mut segment = vec![];

    for id in ctx.src.extra_get(rng) {
      let it: &ast::Type = ctx.src.get(ast::TypeId::new_from(id));

      if let ast::TypeKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
    }

    let (kind, lscp, _span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp).resolve_path(&segment)?.get_k();
    Self::low_resolved(ctx, kind, lscp)
  }

  fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope) -> Result<hir::TypeId, Message> {
    let ty = match kind {
      resolve::ScopeKind::Type(ty) => Self::low(ctx!(lscp -> ctx), ty)?,

      resolve::ScopeKind::TypeParam(thing) => {
        match ctx.src.get(thing) as &ast::Thing {
          ast::Thing::NamedType(_, ty) => Self::low(ctx!(lscp -> ctx), *ty)?,

          ast::Thing::Name(_) => ctx.cre.ty_generic_type(),

          _ => panic!()
        }
      }

      // Expr
      resolve::ScopeKind::Expr(item) => {
        let span = (ctx.src.get(item) as &ast::Item).pos;

        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)))
      }

      // Module
      resolve::ScopeKind::Module(item) => {
        let span = (ctx.src.get(item) as &ast::Item).pos;

        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)))
      }

      kind @_ => panic!("{kind:?}")
    };

    Ok(ty)
  }

  
  fn low_struct(ctx: &mut Ctx, rng: ast::Rng) -> Result<hir::TypeId, Message> {
    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let it: &ast::Item = ctx.src.get(ast::ItemId::new_from(id));
        
        match it.kind {
          ast::ItemKind::Let{kind, ..} => {
            if let Some(kind) = kind {
              let id = Self::low(ctx, kind)?;

              ctn.push(id);
            }
          }

          _ => todo!("{:#?}", it)
        }
      }

      ctx.cre.extra(&ctn)
    };


    // Post
    let this = hir::Type::Struct(
      rng,
    );
    
    Ok(ctx.cre.push(this))
  }

  fn low_tuple(ctx: &mut Ctx, rng: ast::Rng) -> Result<hir::TypeId, Message> {
    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let id = ast::TypeId::new_from(id);
        
        ctn.push(Self::low(ctx, id)?);
      }

      ctx.cre.extra(&ctn)
    };


    // Post
    let this = hir::Type::Struct(
      rng,
    );
    
    Ok(ctx.cre.push(this))
  }

  fn low_fun(ctx: &mut Ctx, rng: ast::Rng, ret: Option<ast::TypeId>) -> Result<hir::TypeId, Message> {
    let args = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let id = ast::TypeId::new_from(id);
        
        ctn.push(Self::low(ctx, id)?);
      }

      ctx.cre.extra(&ctn)
    };

    let ret = match ret {
      None => ctx.cre.ty_unit(),
      Some(kind) => Self::low(ctx, kind)?,
    };


    // Post
    let this = hir::Type::Fun{
      args, ret,
    };
    
    Ok(ctx.cre.push(this))
  }

  fn low_array(ctx: &mut Ctx, kind: ast::TypeId, ext: ast::ExprId) -> Result<hir::TypeId, Message> {
    let kind = TypeLow::low(ctx, kind)?;
    let ext  = ExprLow::low(ctx, ext)?;

    // Post
    let this = hir::Type::Array(
      kind, ext
    );
    
    Ok(ctx.cre.push(this))
  }
  
  fn low_slice(ctx: &mut Ctx, kind: ast::TypeId) -> Result<hir::TypeId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    // Post
    let this = hir::Type::Slice(
      kind,
    );
    
    Ok(ctx.cre.push(this))
  }

}
