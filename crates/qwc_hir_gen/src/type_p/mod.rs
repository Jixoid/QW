/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Attribute};
use qwc_hir::{self as hir, DefPathId, TypeAttr};

use rustc_hash::FxHashMap;
use crate::{Ctx, ExprLow, FunCtx};

mod fun_p;
mod spec_p;
mod enum_p;
mod trait_p;
mod resolve_p;
mod combinated_p;



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
      Unit => ctx.prims.ty_unit,

      // Reference
      Ref(id, ism) => {let id = Self::low(ctx, id)?; ctx.tin.ty_ref(ctx.cre, ctx.ideps, id, ism)},
      
      // Vector
      Vector(kind, ext) => Self::low_vector(ctx, kind, ext)?,
      VScale(kind) => Self::low_vscale(ctx, kind)?,

      // Sequentiel
      Array(kind, ext) => Self::low_array(ctx, kind, ext)?,
      Slice(kind) => Self::low_slice(ctx, kind)?,
      
      // Combinated
      Struct(..) => panic!("use itemty!"),
      Tuple(rng) => {
        let mut path_ids = vec![];
        let elem_ids = {
          let mut ctn = vec![];
          for id in ctx.src.extra_get(rng) {
            let elem_ty = Self::low(ctx, id)?;
            path_ids.push(ctx.get(elem_ty).path);
            ctn.push(elem_ty);
          }
          ctx.cre.extra(&ctn)
        };
        let path_rng = ctx.cre.extra(&path_ids);
        let path = ctx.cre.push(hir::DefPath::AnonymousTuple(path_rng));
        let this = hir::Type {
          kind: hir::TypeKind::Tuple(elem_ids),
          layout: hir::Layout::new_static(hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        };
        ctx.cre.push(this)
      }
      
      // Trait
      Trait(..) => panic!("use itemty!"),
      Iface(..) => panic!("use itemty!"),

      // Enum
      Enum(..)  => panic!("use itemty!"),
      Flags(..) => panic!("use itemty!"),

      // Context
      SelfT => match ctx.cmap.self_ty.last() {
        Some(&v) => return Ok(v),
        None => return Err(Message::error(SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT, Label::new_pos(it.pos)))
      }

      // Function
      Fun{..} => panic!("use itemty!"),

      // Specialize
      Spec{base, args} => spec_p::low_specialize(ctx, it, base, args)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_type.insert(id, it);

    Ok(it)
  }

  pub fn low_itemty(ctx: &mut Ctx, path: DefPathId, id: ast::TypeId) -> Result<hir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }
    
    let it = ctx.src.get(id);

    use ast::TypeKind::*;
  
    let it = match it.kind {
      // Combinated
      Struct(rng) => combinated_p::low_struct(ctx, path, id, rng)?,
      Tuple(rng)   => combinated_p::low_tuple(ctx, path, rng)?,

      // Trait
      Trait(rng) => trait_p::low_trait(ctx, path, rng)?,
      Iface(rng) => trait_p::low_iface(ctx, path, rng)?,

      // Enum
      Enum(rng)  => enum_p::low_enum(ctx, path, rng)?,
      Flags(rng) => enum_p::low_flags(ctx, path, rng)?,

      // Function
      Fun{self_kind, args, ret, ..} => fun_p::low_fun(ctx, path, self_kind, args, ret)?,

      _ => Self::low(ctx, id)?,
    };

    ctx.cmap.cache_type.insert(id, it);

    Ok(it)
  }


  // Vector
  fn low_vector(ctx: &mut Ctx, kind: ast::TypeId, ext: ast::ExprId) -> Result<hir::TypeId, Message> {
    let elem_ty = TypeLow::low(ctx, kind)?;

    // Check
    let lay = ctx.get(elem_ty).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["vector"]), Label::new_pos(pos)));
    }

    let fctx = FunCtx::new_const(ctx.src.get(ext).pos);
    let len_expr = ExprLow::low(ctx, &fctx, ext)?;

    let path = ctx.get(elem_ty).path;
    let this = hir::Type {
      kind: hir::TypeKind::Vector(elem_ty, len_expr),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
      attr: TypeAttr::empty(),
      path,
    };

    Ok(ctx.cre.push(this))
  }
  
  fn low_vscale(ctx: &mut Ctx, kind: ast::TypeId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;

    // Check
    let lay = ctx.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["vscale"]), Label::new_pos(pos)))
    }

    
    // Post
    Ok(ctx.tin.ty_vscale(ctx.cre, ctx.ideps, hir_kind))
  }


  // Sequentiel
  fn low_array(ctx: &mut Ctx, kind: ast::TypeId, ext: ast::ExprId) -> Result<hir::TypeId, Message> {
    let elem_ty = TypeLow::low(ctx, kind)?;

    // Check
    let lay = ctx.get(elem_ty).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["array"]), Label::new_pos(pos)));
    }

    let fctx = FunCtx::new_const(ctx.src.get(ext).pos);
    let len_expr = ExprLow::low(ctx, &fctx, ext)?;

    let path = ctx.get(elem_ty).path;
    let this = hir::Type {
      kind: hir::TypeKind::Array(elem_ty, len_expr),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
      attr: TypeAttr::empty(),
      path,
    };

    Ok(ctx.cre.push(this))
  }
  
  fn low_slice(ctx: &mut Ctx, kind: ast::TypeId) -> Result<hir::TypeId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;

    // Check
    let lay = ctx.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["slice"]), Label::new_pos(pos)))
    }

    
    // Post
    Ok(ctx.tin.ty_slice(ctx.cre, ctx.ideps, hir_kind))
  }

}


fn read_attrs(ctx: &Ctx, attrs: Option<&Vec<Attribute>>) -> Result<TypeAttr, Message> {
  let mut attr = TypeAttr::empty();

  if let Some(attrs) = attrs {
    for key in attrs {
      let key = key.ident;
      
      match () {
        // C
        _ if key.sid() == ctx.sin.sid_c() => {
          if attr.contains(TypeAttr::C) {
            return Err(Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //.add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          attr |= TypeAttr::C;
        }

        _ => return Err(Message::error(UNKNOWN_ATTRIBUTE.args(&[key.str(ctx.far)]), Label::new_pos(key)))
      }
    }
  }

  Ok(attr)
}




pub struct TypeMatch;

impl TypeMatch {

  pub fn matches(ctx: &Ctx, ty1: hir::TypeId, ty2: hir::TypeId, ty2_ast: ast::TypeId, self_ty: Option<hir::TypeId>, assoc_types: Option<&FxHashMap<usize, hir::TypeId>>) -> Result<(), Message> {
    if ty1 == ty2 { return Ok(()) }

    let ty1_ty = ctx.get(ty1).kind;
    let ty2_ty = ctx.get(ty2).kind;
    let ty2_ast_ty = ctx.src.get(ty2_ast).kind;

    if let Some(self_ty) = self_ty {
      if ty1_ty == hir::TypeKind::GenericSelfType {
        if self_ty == ty2 {
          return Ok(());
        } else {
          let ty_pos = ctx.src.get(ty2_ast).pos;
          return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(ty1), &ctx.type_name(ty2)]), Label::new_pos(ty_pos)));
        }
      }
    }

    if let Some(assoc_types) = assoc_types {
      if let hir::TypeKind::AssociatedType { idx, .. } = ty1_ty {
        if let Some(&expected) = assoc_types.get(&idx) {
          if expected == ty2 {
            return Ok(());
          } else {
            return Self::matches(ctx, expected, ty2, ty2_ast, self_ty, Some(assoc_types));
          }
        }
      }
      if let hir::TypeKind::AssociatedType { idx, .. } = ty2_ty {
        if let Some(&expected) = assoc_types.get(&idx) {
          if ty1 == expected {
            return Ok(());
          } else {
            return Self::matches(ctx, ty1, expected, ty2_ast, self_ty, Some(assoc_types));
          }
        }
      }
    }

    match (ty1_ty, ty2_ty) {
      (hir::TypeKind::Error, _) | (_, hir::TypeKind::Error) => Ok(()),
      (_, hir::TypeKind::Never) => Ok(()),

      (hir::TypeKind::Ref(t1, t1_ism), hir::TypeKind::Ref(t2, t2_ism)) if t1_ism == t2_ism => {
        let ast::TypeKind::Ref(t2_ast, _) = ty2_ast_ty else { unreachable!() };

        Self::matches(ctx, t1, t2, t2_ast, self_ty, assoc_types)
      }

      (hir::TypeKind::Ptr(t1, t1_ism), hir::TypeKind::Ptr(t2, t2_ism)) if t1_ism == t2_ism => {
        let ast::TypeKind::Ptr(t2_ast, _) = ty2_ast_ty else { unreachable!() };

        Self::matches(ctx, t1, t2, t2_ast, self_ty, assoc_types)
      }

      _ => {
        let ty_pos = ctx.src.get(ty2_ast).pos;
        return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(ty1), &ctx.type_name(ty2)]), Label::new_pos(ty_pos)));
      }
    }
  }

  pub fn matches_pos(ctx: &Ctx, ty1: hir::TypeId, ty2: hir::TypeId, pos: impl Into<Span>) -> Result<(), Message> {
    if ty1 == ty2 { return Ok(()) }

    let ty1_ty = ctx.get(ty1).kind;
    let ty2_ty = ctx.get(ty2).kind;

    match (ty1_ty, ty2_ty) {
      (hir::TypeKind::Error, _) | (_, hir::TypeKind::Error) => Ok(()),
      (_, hir::TypeKind::Never) => Ok(()),

      (hir::TypeKind::Ref(t1, t1_ism), hir::TypeKind::Ref(t2, t2_ism)) if t1_ism == t2_ism => {
        Self::matches_pos(ctx, t1, t2, pos)
      }

      _ => {
        return Err(Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(ty1), &ctx.type_name(ty2)]), Label::new_pos(pos)));
      }
    }
  }


  pub fn match_fun(ctx: &Ctx, fun1: hir::TypeId, fun2: hir::TypeId, fun2_ast: ast::TypeId, self_ty: Option<hir::TypeId>, assoc_types: Option<&FxHashMap<usize, hir::TypeId>>) -> Result<(), Message> {
    let hir::TypeKind::Fun{self_kind: self_1, args: args_1, ret: ret_1} = ctx.get(fun1).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind: self_2, args: args_2, ret: ret_2} = ctx.get(fun2).kind else { unreachable!() };

    let fun2_ast = ctx.src.get(fun2_ast);
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ret: ret_ast, attr: _} = fun2_ast.kind else { unreachable!() };


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
        let self_1_ty = ctx.get(self_1).kind;
        let self_2_ty = ctx.get(self_2).kind;

        match (self_1_ty, self_2_ty) {
          (hir::TypeKind::Ref(_, ism_1), hir::TypeKind::Ref(_, ism_2)) if ism_1 == ism_2 => {}
          
          _ => TypeMatch::matches(ctx, self_1, self_2, self_ast.unwrap(), self_ty, assoc_types)?,
        }
      }
    }


    // Args Match
    let krate_1 = ctx.get_krate(fun1.cid());
    let krate_2 = ctx.get_krate(fun2.cid());
    let args_1 = krate_1.extra_get(args_1).collect_vec().into_boxed_slice();
    let args_2 = krate_2.extra_get(args_2).collect_vec().into_boxed_slice();
    let args_ast = ctx.src.extra_get(args_ast).collect_vec().into_boxed_slice();
    
    if args_1.len() != args_2.len() {
      return Err(Message::error(
        FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED.args(&[&args_1.len().to_string(), &args_2.len().to_string()]),
        Label::new_pos(fun2_ast.pos),
      ));
    }

    for (id_1, id_2, id_ast) in izip!(args_1, args_2, args_ast) {
      let hir::Thing::NamedType(_, id_1) = *krate_1.get(id_1) else { unreachable!() };
      let hir::Thing::NamedType(_, id_2) = *krate_2.get(id_2) else { unreachable!() };
      let ast::ThingKind::NamedType(_, id_ast) = ctx.src.get(id_ast).kind else { unreachable!() };

      TypeMatch::matches(ctx, id_1, id_2, id_ast, self_ty, assoc_types)?;
    }


    // Ret Match
    ret_ast.map(|ret_ast| TypeMatch::matches(ctx, ret_1, ret_2, ret_ast, self_ty, assoc_types)).transpose()?;
    
    Ok(())
  }

}
