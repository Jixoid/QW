use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast};
use qwc_hir::{self as hir, PushOkApi};

use crate::{Ctx, ExprLow};

mod trait_p;
mod resolve_p;



pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }
    
    let it = ctx.src.get(id);

    use ast::TypeKind::*;
  
    let it = match it.kind {
      // Resolve
      Nick(ident)   => resolve_p::low_nick(ctx, ident)?,
      Path(rng) => resolve_p::low_path(ctx, rng)?,
      
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
      
      // Trait
      Trait(rng) => trait_p::low_trait(ctx, rng)?,
      Iface(rng) => trait_p::low_iface(ctx, rng)?,

      // Context
      SelfT => match ctx.cmap.self_ty.last() {
        Some(&v) => return Ok(v),
        None => return Err(Message::error(SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT, Label::new_pos(it.pos)))
      }

      // Function
      Fun{self_kind, args, ret, ..} => Self::low_fun(ctx, self_kind, args, ret)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_type.insert(id, it);

    Ok(it)
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
  fn low_struct(ctx: &mut Ctx, rng: ast::ThingRng) -> Result<hir::TypeId, Message> {
    let fields_rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        let ast::Thing::NamedType(name, kind) = *ctx.src.get(id) else { panic!() };

        // Check
        let id = Self::low(ctx, kind)?;
        let lay = ctx.cre.get(id).layout;

        if lay.is_dynamic() {
          return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["tuple"]), Label::new_pos(name)))
        }
        
        let id = ctx.cre.push(hir::Thing::NamedType(name.sid(), id));
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


  // Function
  fn low_fun(ctx: &mut Ctx, self_kind: Option<ast::TypeId>, rng: ast::ThingRng, ret: Option<ast::TypeId>) -> Result<hir::TypeId, Message> {
    let args = {
      let mut ctn = vec![];

      for id in ctx.src.extra_get(rng) {
        let ast::Thing::NamedType(name, kind) = *ctx.src.get(id) else { panic!() };

        ctn.push(hir::Thing::NamedType(name.sid(), Self::low(ctx, kind)?).push(ctx.cre));
      }

      ctx.cre.extra(&ctn)
    };

    if let Some(self_kind) = self_kind && ctx.cmap.self_ty.last().is_none() {
      let self_kind = ctx.src.get(self_kind);
      
      return Err(Message::error(SELF_PARAMETER_IS_ONLY_ALLOWED_IN_ASSOCIATED_FUN, Label::new_pos(self_kind.pos)))
    }

    let self_kind = self_kind.map(|id| Self::low(ctx, id)).transpose()?;

    let ret = match ret {
      None => ctx.tin.ty_unit(),
      Some(kind) => Self::low(ctx, kind)?,
    };


    // Post
    let this = hir::Type {
      kind: hir::TypeKind::Fun{self_kind, args, ret},
      layout: hir::Layout::new_dst(hir::LayoutBy::QW),
    };
    
    Ok(ctx.cre.push(this))
  }

}



pub struct TypeMatch;

impl TypeMatch {

  pub fn matches(ctx: &Ctx, ty1: hir::TypeId, ty2: hir::TypeId, ty2_ast: ast::TypeId) -> Result<(), Message> {
    if ty1 == ty2 { return Ok(()) }

    let ty1_ty = ctx.cre.get(ty1).kind;
    let ty2_ty = ctx.cre.get(ty2).kind;
    let ty2_ast_ty = ctx.src.get(ty2_ast).kind;

    match (ty1_ty, ty2_ty) {
      (hir::TypeKind::Ref(t1, t1_ism), hir::TypeKind::Ref(t2, t2_ism)) if t1_ism == t2_ism => {
        let ast::TypeKind::Ref(t2_ast, _) = ty2_ast_ty else { panic!() };

        Self::matches(ctx, t1, t2, t2_ast)
      }

      _ => {
        let ty_pos = ctx.src.get(ty2_ast).pos;
        return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(ty1), &ctx.type_name(ty2)]), Label::new_pos(ty_pos)));
      }
    }
  }

  pub fn matches_pos(ctx: &Ctx, ty1: hir::TypeId, ty2: hir::TypeId, pos: impl Into<Span>) -> Result<(), Message> {
    if ty1 == ty2 { return Ok(()) }

    let ty1_ty = ctx.cre.get(ty1).kind;
    let ty2_ty = ctx.cre.get(ty2).kind;

    match (ty1_ty, ty2_ty) {
      (hir::TypeKind::Ref(t1, t1_ism), hir::TypeKind::Ref(t2, t2_ism)) if t1_ism == t2_ism => {
        Self::matches_pos(ctx, t1, t2, pos)
      }

      _ => {
        return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(ty1), &ctx.type_name(ty2)]), Label::new_pos(pos)));
      }
    }
  }


  pub fn match_fun(ctx: &Ctx, fun1: hir::TypeId, fun2: hir::TypeId, fun2_ast: ast::TypeId) -> Result<(), Message> {
    let hir::TypeKind::Fun{self_kind: self_1, args: args_1, ret: ret_1} = ctx.cre.get(fun1).kind else { panic!() };
    let hir::TypeKind::Fun{self_kind: self_2, args: args_2, ret: ret_2} = ctx.cre.get(fun2).kind else { panic!() };

    let fun2_ast = ctx.src.get(fun2_ast);
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ret: ret_ast, attr: _} = fun2_ast.kind else { panic!() };


    // Self Match
    match (self_1, self_2) {
      (None, None) => {},

      (None, Some(..)) => {
        let self_pos = ctx.src.get(self_ast.unwrap()).pos;
        return Err(Message::error(INCOMPATIBLE_METHOD, Label::new(self_pos, UNEXPECTED_PARAMETER)))
      }

      (Some(..), None) => {
        return Err(Message::error(INCOMPATIBLE_METHOD, Label::new(fun2_ast.pos, ARGUMENT_X_IS_MISSING.args(&["self"]))))
      }

      (Some(self_1), Some(self_2)) => {
        let self_1_ty = ctx.cre.get(self_1).kind;
        let self_2_ty = ctx.cre.get(self_2).kind;

        match (self_1_ty, self_2_ty) {
          (hir::TypeKind::Ref(_, ism_1), hir::TypeKind::Ref(_, ism_2)) if ism_1 == ism_2 => {}
          
          _ => TypeMatch::matches(ctx, self_1, self_2, self_ast.unwrap())?,
        }
      }
    }


    // Args Match
    let args_1 = ctx.cre.extra_get(args_1).collect_vec().into_boxed_slice();
    let args_2 = ctx.cre.extra_get(args_2).collect_vec().into_boxed_slice();
    let args_ast = ctx.src.extra_get(args_ast).collect_vec().into_boxed_slice();
    
    if args_1.len() != args_2.len() {
      return Err(Message::error(
        FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED.args(&[&args_1.len().to_string(), &args_2.len().to_string()]),
        Label::new_pos(fun2_ast.pos),
      ));
    }

    for (id_1, id_2, id_ast) in izip!(args_1, args_2, args_ast) {
      let hir::Thing::NamedType(_, id_1) = *ctx.cre.get(id_1) else { unreachable!() };
      let hir::Thing::NamedType(_, id_2) = *ctx.cre.get(id_2) else { unreachable!() };
      let ast::Thing::NamedType(_, id_ast) = *ctx.src.get(id_ast) else { unreachable!() };

      TypeMatch::matches(ctx, id_1, id_2, id_ast)?;
    }


    // Ret Match
    ret_ast.map(|ret_ast| TypeMatch::matches(ctx, ret_1, ret_2, ret_ast)).transpose()?;
    
    Ok(())
  }

}
