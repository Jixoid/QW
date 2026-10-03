use qwc_diagnostic::{Label, Message, msg::*};
use qwc_ast::{self as ast, Ident};
use qwc_hir as hir;
use qwc_resolve::{self as resolve, Resolver};

use crate::{Ctx, ExprLow, ctx};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }
    
    let it = ctx.src.get(id);

    use ast::TypeKind::*;
  
    let it = match it.kind {
      // Resolve
      Nick(ident)   => Self::low_nick(ctx, ident)?,
      Path(rng) => Self::low_path(ctx, rng)?,
      
      // Basic
      Unit => ctx.tin.ty_unit(),

      // Reference
      Ref(id, ism) => {let id = Self::low(ctx, id)?; ctx.tin.ty_ref(ctx.cre, id, ism)},
      
      // Vector
      Vector(kind, ext) => Self::low_vector(ctx, kind, ext)?,
      VScale(kind) => Self::low_vscale(ctx, kind)?,

      // Sequentiel
      Array(kind, ext) => Self::low_array(ctx, kind, ext)?,
      Slice(kind) => Self::low_slice(ctx, kind)?,
      
      // Combinated
      Struct(rng) => Self::low_struct(ctx, rng)?,
      Tuple(rng)  => Self::low_tuple(ctx, rng)?,
      
      // Interface
      Iface(rng) => Self::low_iface(ctx, rng)?,

      // Context
      SelfT => return Ok(ctx.cmap.self_ty.unwrap()),

      // Function
      Fun{self_kind, args, ret, ..} => Self::low_fun(ctx, self_kind, args, ret)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_type.insert(id, it);

    Ok(it)
  }


  // Resolve
  fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::TypeId, Message> {
    let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).lookup(ident)?.get_k();
    Self::low_resolved(ctx, kind, lscp, span)
  }

  fn low_path(ctx: &mut Ctx, rng: ast::TypeRng) -> Result<hir::TypeId, Message> {
    let mut segment = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

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
          let span = ctx.src.get(item).pos;

          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
        }

        // Module
        resolve::ScopeKindAst::Module(item) => {
          let span = ctx.src.get(item).pos;

          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "module"]), Label::new_pos(span)))
        }

        resolve::ScopeKindAst::ExprParam(..) | resolve::ScopeKindAst::Local(..) => panic!("value in type position"),
      },

      resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
        resolve::ScopeKindHir::Type(ty) => ctx.low_hir_type(ty),

        resolve::ScopeKindHir::Expr(..) | resolve::ScopeKindHir::Module(..) => {
          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
        }
      }
    };

    Ok(ty)
  }


  // Vector
  fn low_vector(ctx: &mut Ctx, kind: ast::TypeId, ext: ast::ExprId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    let hir_ext  = ExprLow::low(ctx, ext)?;

    // Check
    let lay = ctx.cre.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;
    
    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["vector"]), Label::new_pos(pos)))
    }

    
    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Vector(hir_kind, hir_ext),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }
  
  fn low_vscale(ctx: &mut Ctx, kind: ast::TypeId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;

    // Check
    let lay = ctx.cre.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["vscale"]), Label::new_pos(pos)))
    }

    
    // Post
    Ok(ctx.tin.ty_vscale(ctx.cre, hir_kind))
  }


  // Sequentiel
  fn low_array(ctx: &mut Ctx, kind: ast::TypeId, ext: ast::ExprId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    let hir_ext  = ExprLow::low(ctx, ext)?;

    // Check
    let lay = ctx.cre.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;
    
    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["array"]), Label::new_pos(pos)))
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
    let lay = ctx.cre.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["slice"]), Label::new_pos(pos)))
    }

    
    // Post
    Ok(ctx.tin.ty_slice(ctx.cre, hir_kind))
  }


  // Combinated
  fn low_struct(ctx: &mut Ctx, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
    let fields_rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let it = ctx.src.get(id);
        
        let id = match it.kind {
          ast::FieldKind::MemberVar{kind} => Self::low(ctx, kind)?,
          ast::FieldKind::ImplIn{..} | ast::FieldKind::Fun{..} => continue,
          _ => todo!("{:#?}", it)
        };

        // Check
        let lay = ctx.cre.get(id).layout;

        if lay.is_dynamic() {
          return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["struct"]), Label::new_pos(it.pos)))
        }

        let id = ctx.cre.push(hir::Thing::NamedType(it.name.unwrap().sid(), id));
        ctn.push(id);
      }

      ctx.cre.extra(&ctn)
    };

    let this = hir::Type {
      kind: hir::TypeKind::Struct(fields_rng),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
    };

    Ok(ctx.cre.push(this))
  }

  fn low_tuple(ctx: &mut Ctx, rng: ast::TypeRng) -> Result<hir::TypeId, Message> {
    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        // Check
        let pos = ctx.src.get(id).pos;
        let id = Self::low(ctx, id)?;
        let lay = ctx.cre.get(id).layout;

        if lay.is_dynamic() {
          return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["tuple"]), Label::new_pos(pos)))
        }
        
        ctn.push(id);
      }

      ctx.cre.extra(&ctn)
    };


    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Tuple(rng),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }


  // Interface
  fn low_iface(ctx: &mut Ctx, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
    let iface_id = ctx.cre.push(hir::Type {
      kind: hir::TypeKind::Iface(hir::Rng::empty()),
      layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    });

    let prev_self = ctx.cmap.self_ty;
    ctx.cmap.self_ty = Some(iface_id);

    let methods = {
      let mut ctn = vec![];
      for id in ctx.src.extra_get(rng) {
        let it = ctx.src.get(id);
        match it.kind {
          ast::FieldKind::Fun { kind, .. } => {
            let fun_ast = ctx.src.get(kind);
            let ast::TypeKind::Fun { self_kind, .. } = fun_ast.kind else { unreachable!() };

            let fun_ty = match Self::low(ctx, kind) {
              Ok(ty) => ty,
              Err(err) => {
                ctx.cmap.self_ty = prev_self;
                return Err(err);
              }
            };

            if let Some(self_id) = self_kind {
              let hir::TypeKind::Fun { args, .. } = ctx.cre.get(fun_ty).kind else { unreachable!() };
              let self_hir_ty = ctx.cre.extra_get(args).next().unwrap();
              let lay = ctx.cre.get(self_hir_ty).layout;
              if !lay.is_static() {
                ctx.cmap.self_ty = prev_self;
                let pos = ctx.src.get(self_id).pos;
                return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["iface"]), Label::new_pos(pos)));
              }
            }

            let id = ctx.cre.push(hir::Thing::NamedType(it.name.unwrap().sid(), fun_ty));
            ctn.push(id);
          }
          _ => {}
        }
      }
      ctx.cre.extra(&ctn)
    };

    ctx.cmap.self_ty = prev_self;

    let this = ctx.cre.get_mut(iface_id);
    this.kind = hir::TypeKind::Iface(methods);

    Ok(iface_id)
  }

  
  // Function
  fn low_fun(ctx: &mut Ctx, self_kind: Option<ast::TypeId>, rng: ast::ThingRng, ret: Option<ast::TypeId>) -> Result<hir::TypeId, Message> {
    let args = {
      let mut ctn = vec![];

      if let Some(self_id) = self_kind {
        ctn.push(Self::low(ctx, self_id)?);
      }
      
      for id in ctx.src.extra_get(rng) {
        let it = ctx.src.get(id);

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

}
