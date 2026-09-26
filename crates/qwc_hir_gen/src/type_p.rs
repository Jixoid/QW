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

    use ast::TypeKind::*;
  
    let it = match it.kind {
      // Resolve
      Nick(ident) => Self::low_nick(ctx, ident)?,
      Path(rng)     => Self::low_path(ctx, rng)?,
      
      // ZST
      Unit => ctx.tin.ty_unit(),

      // Combinated
      Struct(rng) => Self::low_struct(ctx, rng)?,
      Tuple(rng)  => Self::low_tuple(ctx, rng)?,
      Slice(kind) => Self::low_slice(ctx, kind)?,
      
      // Sequentiel
      Array(kind, ext) => Self::low_array(ctx, kind, ext)?,
      
      // Callable
      Fun{args, ret, ..} => Self::low_fun(ctx, args, ret)?,
      
      // Reference
      Ref(id, ism) => {let id = Self::low(ctx, id)?; ctx.tin.ty_ref(ctx.cre, id, ism)},

      // RT contract
      Trait(rng) => Self::low_trait(ctx, rng)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_type.insert(id, it);

    Ok(it)
  }


  fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::TypeId, Message> {
    let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).lookup(ident)?.get_k();
    Self::low_resolved(ctx, kind, lscp, span)
  }

  fn low_path(ctx: &mut Ctx, rng: ast::TypeRng) -> Result<hir::TypeId, Message> {
    let mut segment = vec![];

    for id in ctx.src.extra_get(rng) {
      let it: &ast::Type = ctx.src.get(id);

      if let ast::TypeKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
    }

    let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(&segment)?.get_k();
    Self::low_resolved(ctx, kind, lscp, span)
  }

  fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope, span: qwc_diagnostic::Span) -> Result<hir::TypeId, Message> {
    let ty = match kind {
      resolve::ScopeKind::Ast(ast_kind) => match ast_kind {
        resolve::ScopeKindAst::Type(ty) => Self::low(ctx!(lscp -> ctx), ty)?,

        resolve::ScopeKindAst::TypeParam(thing) => {
          match ctx.src.get(thing) as &ast::Thing {
            ast::Thing::NamedType(_, ty) => Self::low(ctx!(lscp -> ctx), *ty)?,

            ast::Thing::Name(_) => ctx.tin.ty_generic_type(),

            _ => panic!()
          }
        }

        // Expr
        resolve::ScopeKindAst::Expr(item) => {
          let span = (ctx.src.get(item) as &ast::Item).pos;

          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)))
        }

        // Module
        resolve::ScopeKindAst::Module(item) => {
          let span = (ctx.src.get(item) as &ast::Item).pos;

          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)))
        }

        resolve::ScopeKindAst::ExprParam(..) | resolve::ScopeKindAst::Local(..) => panic!("value in type position"),
      },

      resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
        resolve::ScopeKindHir::Type(ty) => ctx.low_hir_type(ty),

        resolve::ScopeKindHir::Expr(..) | resolve::ScopeKindHir::Module(..) => {
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)))
        }
      }
    };

    Ok(ty)
  }

  
  fn low_struct(ctx: &mut Ctx, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let it: &ast::Field = ctx.src.get(id);
        
        let id = match it.kind {
          ast::FieldKind::MemberVar{kind} => Self::low(ctx, kind)?,
          _ => todo!("{:#?}", it)
        };

        // Check
        let lay = (ctx.cre.get(id) as &hir::Type).layout;

        match lay.kind() {
          hir::LayoutKind::Static => (),

          // Unsupported DST
          hir::LayoutKind::DST | hir::LayoutKind::DSAT => return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_STRUCT, Label::new_pos(it.pos)))
        }

        ctn.push(id);
      }

      ctx.cre.extra(&ctn)
    };


    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Struct(rng),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }

  fn low_tuple(ctx: &mut Ctx, rng: ast::TypeRng) -> Result<hir::TypeId, Message> {
    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        // Check
        let pos = (ctx.src.get(id) as &ast::Type).pos;
        let id = Self::low(ctx, id)?;
        let lay = (ctx.cre.get(id) as &hir::Type).layout;

        match lay.kind() {
          hir::LayoutKind::Static => (),

          // Unsupported DST
          hir::LayoutKind::DST | hir::LayoutKind::DSAT => return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_TUPLE, Label::new_pos(pos)))
        }
        
        ctn.push(id);
      }

      ctx.cre.extra(&ctn)
    };


    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Struct(rng),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }

  
  fn low_fun(ctx: &mut Ctx, rng: ast::ThingRng, ret: Option<ast::TypeId>) -> Result<hir::TypeId, Message> {
    let args = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let it: &ast::Thing = ctx.src.get(id);

        let ty = match *it {
          ast::Thing::NamedType(_, ty) => ty,

          _ => panic!()
        };

        ctn.push(Self::low(ctx, ty)?);
      }

      ctx.cre.extra(&ctn)
    };

    let ret = match ret {
      None => ctx.tin.ty_unit(),
      Some(kind) => Self::low(ctx, kind)?,
    };


    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Fun{args, ret},
      layout: hir::Layout::new_dst(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }


  fn low_array(ctx: &mut Ctx, kind: ast::TypeId, ext: ast::ExprId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    let hir_ext  = ExprLow::low(ctx, ext)?;

    // Check
    let lay = (ctx.cre.get(hir_kind) as &hir::Type).layout;
    let pos = (ctx.src.get(kind) as &ast::Type).pos;
    
    match lay.kind() {
      hir::LayoutKind::Static => (),

      // Unsupported DST
      hir::LayoutKind::DST | hir::LayoutKind::DSAT => return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_ARRAY, Label::new_pos(pos)))
    }

    
    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Array(hir_kind, hir_ext),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }
  
  fn low_slice(ctx: &mut Ctx, kind: ast::TypeId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;

    // Check
    let lay = (ctx.cre.get(hir_kind) as &hir::Type).layout;
    let pos = (ctx.src.get(kind) as &ast::Type).pos;

    match lay.kind() {
      hir::LayoutKind::Static => (),

      // Unsupported DST
      hir::LayoutKind::DST | hir::LayoutKind::DSAT => return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_ARRAY, Label::new_pos(pos)))
    }

    
    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Slice(hir_kind),
      layout: hir::Layout::new_static(hir::LayoutBy::QW)
    };
    
    Ok(ctx.cre.push(this))
  }


  fn low_trait(_ctx: &mut Ctx, _rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
    //let mut funs = vec![];

    //for id in ctx.src.extra_get(rng) {
    //  let id = FieldLow::low(ctx, id)?.unwrap();
    //  let it: &hir::Item = ctx.cre.get(id);
    //
    //  match it.kind {
    //    hir::ItemKind::Function{..} => funs.push(id),
    //
    //    _ => todo!("{:#?}", it.kind)
    //  }
    //}
    

    todo!()
  }

}
