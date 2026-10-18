/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::{Itertools, izip};
use qwc_comptime::{Evaluator, GetConst};
use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_ast as ast;
use qwc_hir::{self as hir, PushOkApi, Span, TypeAttr};

use rustc_hash::FxHashMap;
use crate::{Ctx, ExprLow, FunCtx};

mod fun_p;
mod spec_p;
mod enum_p;
mod helper;
mod trait_p;
mod resolve_p;
mod variant_p;
mod combinated_p;



pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, sum: &mut Summary, id: ast::TypeId) -> Status<hir::TypeId> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Status::Success(id) }
    
    let it = ctx.src.get(id);

    use ast::TypeKind::*;
    let it = match it.kind {
      // Resolve
      Nick(ident)   => resolve_p::low_nick(ctx, sum, it, ident)?,
      Path(rng) => resolve_p::low_path(ctx, sum, it, rng)?,
      
      // Basic
      Unit => ctx.prims.ty_unit,

      // Reference
      Ref(id, ism) => {let id = Self::low(ctx, sum, id)?; ctx.tin.ty_ref(ctx.cre, ctx.ideps, id, ism)},
      
      // Vector
      Vector(kind, ext) => Self::low_vector(ctx, sum, kind, ext)?,
      VScale(kind) => Self::low_vscale(ctx, sum, kind)?,

      // Sequentiel
      Array(kind, ext) => Self::low_array(ctx, sum, kind, ext)?,
      Slice(kind) => Self::low_slice(ctx, sum, kind)?,
      
      // Combinated
      Struct {vars, path} => combinated_p::low_struct(ctx, sum, id,  path, vars)?,
      Tuple {vars, path}   => combinated_p::low_tuple(ctx, sum, path, vars)?,
      
      TupleAnon(rng) => combinated_p::low_tuple_anon(ctx, sum, rng)?,
      
      Range(kind) => Self::low_range(ctx, sum, kind)?,

      // Trait
      Trait {fields, reqs, path} => trait_p::low_trait(ctx, sum, path, fields, reqs)?,
      Iface {fields, reqs, path} => trait_p::low_iface(ctx, sum, path, fields, reqs)?,

      // Enum
      Enum  {fields, path} => enum_p::low_enum(ctx, sum, path, fields)?,
      Flags {fields, path} => enum_p::low_flags(ctx, sum, path, fields)?,

      // Variant
      Option(kind) => variant_p::low_option(ctx, sum, kind)?,

      // Context
      SelfT => match ctx.cmap.self_ty.last() {
        Some(&v) => return Status::Success(v),
        None => ret_err!(sum, Message::error(SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT, Label::new_pos(it.pos))),
      }

      // Function
      Fun{self_kind, args, ret, ..} => fun_p::low_fun(ctx, sum, ctx.path, self_kind, args, ret)?,

      // Specialize
      Spec{base, args} => spec_p::low_specialize(ctx, sum, it, base, args)?,
      
      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_type.insert(id, it);

    Status::Success(it)
  }


  // Vector
  fn low_vector(ctx: &mut Ctx, sum: &mut Summary, kind: ast::TypeId, ext: ast::ExprId) -> Status<hir::TypeId> {
    let elem_ty = TypeLow::low(ctx, sum, kind)?;

    // Check
    let lay = ctx.get(elem_ty).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["vector"]), Label::new_pos(pos)));
    }

    let fctx = FunCtx::new_const(ctx.src.get(ext).pos);
    let len_expr = ExprLow::low(ctx, &fctx, sum, ext)?;

    let path = ctx.get(elem_ty).path;


    // Post
    hir::Type {
      kind: hir::TypeKind::Vector(elem_ty, len_expr),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
      attr: TypeAttr::empty(),
      path,
    }.push_ok(ctx.cre)
  }
  
  fn low_vscale(ctx: &mut Ctx, sum: &mut Summary, kind: ast::TypeId) -> Status<hir::TypeId> {
    let hir_kind = TypeLow::low(ctx, sum, kind)?;

    // Check
    let lay = ctx.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["vscale"]), Label::new_pos(pos)));
    }

    
    // Post
    Status::Success(ctx.tin.ty_vscale(ctx.cre, ctx.ideps, hir_kind))
  }


  // Sequentiel
  fn low_array(ctx: &mut Ctx, sum: &mut Summary, kind: ast::TypeId, ext: ast::ExprId) -> Status<hir::TypeId> {
    let elem_ty = TypeLow::low(ctx, sum, kind)?;

    // Check
    let lay = ctx.get(elem_ty).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["array"]), Label::new_pos(pos)));
    }

    let fctx = FunCtx::new_const(ctx.src.get(ext).pos);
    let len_expr = ExprLow::low(ctx, &fctx, sum, ext)?;

    let path = ctx.get(elem_ty).path;


    // Post
    hir::Type {
      kind: hir::TypeKind::Array(elem_ty, len_expr),
      layout: hir::Layout::new_static(hir::LayoutBy::QW),
      attr: TypeAttr::empty(),
      path,
    }.push_ok(ctx.cre)
  }
  
  fn low_slice(ctx: &mut Ctx, sum: &mut Summary, kind: ast::TypeId) -> Status<hir::TypeId> {
    let hir_kind = TypeLow::low(ctx, sum, kind)?;

    // Check
    let lay = ctx.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["slice"]), Label::new_pos(pos)));
    }

    
    // Post
    Status::Success(ctx.tin.ty_slice(ctx.cre, ctx.ideps, hir_kind))
  }

  fn low_range(ctx: &mut Ctx, sum: &mut Summary, kind: ast::TypeId) -> Status<hir::TypeId> {
    let hir_kind = TypeLow::low(ctx, sum, kind)?;

    // Check
    let lay = ctx.get(hir_kind).layout;
    let pos = ctx.src.get(kind).pos;

    if lay.is_dynamic() {
      ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["range"]), Label::new_pos(pos)));
    }

    
    // Post
    Status::Success(ctx.tin.ty_range(ctx.cre, ctx.ideps, hir_kind))
  }

}


pub struct TypeMatch;

impl TypeMatch {

  pub fn matches(ctx: &Ctx, sum: &mut Summary, ty1: hir::TypeId, ty2: hir::TypeId, pos: Span, self_ty: Option<hir::TypeId>, assoc_types: Option<&FxHashMap<usize, hir::TypeId>>) -> Status<()> {
    if ty1 == ty2 { return Status::Success(()) }

    let ty1_ty = ctx.get(ty1).kind;
    let ty2_ty = ctx.get(ty2).kind;

    if let Some(self_ty) = self_ty {
      if ty1_ty == hir::TypeKind::GenericSelfType {
        if self_ty == ty2 {
          return Status::Success(());
        } else {
          ret_err!(sum, Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(sum, ty1), &ctx.type_name(sum, ty2)]), Label::new_pos(pos)));
        }
      }
    }

    if let Some(assoc_types) = assoc_types {
      if let hir::TypeKind::AssociatedType { idx, .. } = ty1_ty {
        if let Some(&expected) = assoc_types.get(&idx) {
          if expected == ty2 {
            return Status::Success(());
          } else {
            return Self::matches(ctx, sum, expected, ty2, pos, self_ty, Some(assoc_types));
          }
        }
      }
      if let hir::TypeKind::AssociatedType { idx, .. } = ty2_ty {
        if let Some(&expected) = assoc_types.get(&idx) {
          if ty1 == expected {
            return Status::Success(());
          } else {
            return Self::matches(ctx, sum, ty1, expected, pos, self_ty, Some(assoc_types));
          }
        }
      }
    }

    match (ty1_ty, ty2_ty) {
      (hir::TypeKind::Error, _) | (_, hir::TypeKind::Error) => Status::Success(()),
      (_, hir::TypeKind::Never) => Status::Success(()),

      (hir::TypeKind::Ref(t1, t1_ism), hir::TypeKind::Ref(t2, t2_ism)) if t1_ism == t2_ism => {
        Self::matches(ctx, sum, t1, t2, pos, self_ty, assoc_types)
      }

      (hir::TypeKind::Ptr(t1, t1_ism), hir::TypeKind::Ptr(t2, t2_ism)) if t1_ism == t2_ism => {
        Self::matches(ctx, sum, t1, t2, pos, self_ty, assoc_types)
      }

      (hir::TypeKind::Slice(t1), hir::TypeKind::Slice(t2)) => {
        Self::matches(ctx, sum, t1, t2, pos, self_ty, assoc_types)
      }

      (hir::TypeKind::Array(t1, l1), hir::TypeKind::Array(t2, l2)) => {
        let eval = Evaluator::new(ctx.cre);

        let len1 = eval.eval(sum, l1)?.get_u64(sum)?;
        let len2 = eval.eval(sum, l2)?.get_u64(sum)?;

        if l1 == l2 || len1 == len2 {
          Self::matches(ctx, sum, t1, t2, pos, self_ty, assoc_types)
        } else {
          ret_err!(sum, Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(sum, ty1), &ctx.type_name(sum, ty2)]), Label::new_pos(pos)));
        }
      }

      _ => ret_err!(sum, Message::error(MISMATCHED_TYPES.args(&[&ctx.type_name(sum, ty1), &ctx.type_name(sum, ty2)]), Label::new_pos(pos))),
    }
  }


  pub fn match_fun(ctx: &Ctx, sum: &mut Summary, fun1: hir::TypeId, fun2: hir::TypeId, pos: Span, self_ty: Option<hir::TypeId>, assoc_types: Option<&FxHashMap<usize, hir::TypeId>>) -> Status<()> {
    let hir::TypeKind::Fun{self_kind: self_1, args: args_1, ret: ret_1} = ctx.get(fun1).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind: self_2, args: args_2, ret: ret_2} = ctx.get(fun2).kind else { unreachable!() };

    // Self Match
    match (self_1, self_2) {
      (None, None) => {},

      (None, Some(..)) => {
        ret_err!(sum, Message::error(INCOMPATIBLE_METHOD, Label::new(pos, UNEXPECTED_PARAMETER)))
      }

      (Some(..), None) => {
        ret_err!(sum, Message::error(INCOMPATIBLE_METHOD, Label::new(pos, ARGUMENT_X_IS_MISSING.args(&["self"]))))
      }

      (Some(self_1), Some(self_2)) => {
        let self_1_ty = ctx.get(self_1).kind;
        let self_2_ty = ctx.get(self_2).kind;

        match (self_1_ty, self_2_ty) {
          (hir::TypeKind::Ref(_, ism_1), hir::TypeKind::Ref(_, ism_2)) if ism_1 == ism_2 => {}
          
          _ => TypeMatch::matches(ctx, sum, self_1, self_2, pos, self_ty, assoc_types)?,
        }
      }
    }


    // Args Match
    let krate_1 = ctx.get_krate(fun1.cid());
    let krate_2 = ctx.get_krate(fun2.cid());
    let args_1 = krate_1.extra_get(args_1).collect_vec();
    let args_2 = krate_2.extra_get(args_2).collect_vec();
    
    if args_1.len() != args_2.len() {
      ret_err!(sum, Message::error(FUNCTION_TAKES_X_ARGUMENTS_BUT_X_WERE_SUPPLIED
        .args(&[
          &args_1.len().to_string(),
          &args_2.len().to_string(),
        ]),
        Label::new_pos(pos),
      ));
    }

    for (id_1, id_2) in izip!(args_1, args_2) {
      let hir::Thing::NamedType(_, id_1) = *krate_1.get(id_1) else { unreachable!() };
      let hir::Thing::NamedType(_, id_2) = *krate_2.get(id_2) else { unreachable!() };

      TypeMatch::matches(ctx, sum, id_1, id_2, pos, self_ty, assoc_types)?;
    }


    // Ret Match
    TypeMatch::matches(ctx, sum, ret_1, ret_2, pos, self_ty, assoc_types)?;
    
    Status::Success(())
  }

}
