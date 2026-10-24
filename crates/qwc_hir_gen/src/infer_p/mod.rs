/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, Status::{self, Fail, Success}, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_string_interner::Sid;

use qwc_ast as ast;
use crate::{Ctx, ItemLow, TypeMatch};


pub struct Infer;

impl Infer {

  pub fn resolve_method(ctx: &mut Ctx, sum: &mut Summary, this: hir::ExprId, callee: hir::ExprId, user_args: &[hir::ExprId], pos: impl Into<Span>) -> Status<hir::ExprId> {
    let this_ty = ctx.cre.get(this).ety;

    let method_name = if let hir::ExprKind::GlobalRef(item) = ctx.cre.get(callee).kind {
      ctx.cre.get(item).name
    } else {
      None
    };

    let Some(method_name) = method_name else { return Status::Success(callee); };
    let Some(type_methods) = ctx.type_impls.get(&this_ty) else { return Status::Success(callee); };
    let Some(cand_impl_items) = type_methods.methods.get(&method_name) else { return Status::Success(callee); };

    if cand_impl_items.len() <= 1 {
      return Status::Success(callee);
    }

    let pos_span = pos.into();
    let mut matched = vec![];

    for &impl_ast_id in cand_impl_items {
      if let Success(Some(impl_hir_id)) = ItemLow::low(ctx, sum, impl_ast_id) {
        let hir::ItemKind::Impl { methods, .. } = ctx.cre.get(impl_hir_id).kind else { continue; };
        let Some(cand_method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name == Some(method_name)) else { continue; };
        let hir::ItemKind::Function { kind: cand_fn_ty, .. } = ctx.cre.get(cand_method).kind else { continue; };
        let hir::TypeKind::Fun { self_kind: cand_self, args: cand_params, .. } = ctx.get(cand_fn_ty).kind else { continue; };

        if let Some(cand_self_ty) = cand_self {
          if TypeMatch::matches(ctx, sum, cand_self_ty, this_ty, ctx.get(this).pos, None).is_fail() {
            continue;
          }
        }

        if cand_params.count() as usize != user_args.len() {
          continue;
        }

        let mut all_match = true;
        for (&user_arg, cand_param) in user_args.iter().zip(ctx.extra_get(cand_params)) {
          let hir::Thing::NamedType(_, cand_param_ty) = *ctx.get(cand_param) else { unreachable!() };
          let user_ty = ctx.cre.get(user_arg).ety;
          if TypeMatch::matches(ctx, sum, cand_param_ty, user_ty, ctx.get(user_arg).pos, None).is_fail() {
            all_match = false;
            break;
          }
        }

        if all_match {
          matched.push((cand_method, cand_fn_ty));
        }
      }
    }

    if matched.len() == 1 {
      let (matched_method, matched_fn_ty) = matched[0];
      Status::Success(hir::Expr {
        kind: hir::ExprKind::GlobalRef(matched_method),
        category: ExprCategory::RValue,
        pos: pos_span,
        ety: matched_fn_ty,
      }.push(ctx.cre))
    } else if matched.len() > 1 {
      ret_err!(sum, Message::error(AMBIGUOUS_METHOD_CALL, Label::new_pos(pos_span)))
    } else {
      Status::Success(callee)
    }
  }

  pub fn lookup_method(ctx: &mut Ctx, sum: &mut Summary, current_id: hir::ExprId, sid: Sid) -> Status<hir::ExprId> {
    let this_ty = ctx.cre.get(current_id).ety;
    if let Some(methods) = ctx.type_impls.get(&this_ty) {
      if let Some(items) = methods.methods.get(&sid) {
        for &item in items {
          if let Success(Some(impl_id)) = ItemLow::low(ctx, sum, item) {
            let hir::ItemKind::Impl { methods, .. } = ctx.cre.get(impl_id).kind else { unreachable!() };
            let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue };
            let hir::ItemKind::Function { kind, .. } = ctx.cre.get(method).kind else { unreachable!() };

            let callee = hir::Expr {
              kind: hir::ExprKind::GlobalRef(method),
              category: ExprCategory::RValue,
              ety: kind,
              pos: ctx.cre.get(current_id).pos,
            }.push(ctx.cre);

            return hir::Expr {
              kind: hir::ExprKind::BoundSelfMethod {
                callee,
                this: current_id,
              },
              category: ExprCategory::RValue,
              ety: ctx.prims.ty_unit,
              pos: ctx.cre.get(current_id).pos,
            }.push_ok(ctx.cre);
          }
        }
      }
    }

    for &cid in ctx.imods {
      let dep_krate = ctx.ideps.get(cid);
      for impl_id in dep_krate.impl_items() {
        let hir::ItemKind::Impl { type_ty, methods, .. } = ctx.get(impl_id).kind else { continue; };
        if type_ty != this_ty {
          continue;
        }
        let Some(method) = ctx.extra_get(methods).find(|&m| ctx.get(m).name == Some(sid)) else { continue; };
        let hir::ItemKind::Function { kind, .. } = ctx.get(method).kind else { continue; };

        let callee = hir::Expr {
          kind: hir::ExprKind::GlobalRef(method),
          category: ExprCategory::RValue,
          ety: kind,
          pos: ctx.cre.get(current_id).pos,
        }.push(ctx.cre);

        return hir::Expr {
          kind: hir::ExprKind::BoundSelfMethod {
            callee,
            this: current_id,
          },
          category: ExprCategory::RValue,
          ety: ctx.prims.ty_unit,
          pos: ctx.cre.get(current_id).pos,
        }.push_ok(ctx.cre);
      }
    }

    hir::Expr {
      kind: hir::ExprKind::Error,
      category: ExprCategory::RValue,
      ety: ctx.prims.ty_error,
      pos: ctx.cre.get(current_id).pos,
    }.push_ok(ctx.cre)
  }

  pub fn lookup_static_method(ctx: &mut Ctx, sum: &mut Summary, target_ty: hir::TypeId, sid: Sid, pos: Span) -> Status<hir::ExprId> {
    let target_unraw_ty = match ctx.get(target_ty).kind {
      hir::TypeKind::GenericRaw { kind, .. } => kind,
      _ => target_ty,
    };
    let found_const = {
      let krate = ctx.get_krate(target_unraw_ty.cid());
      if let hir::TypeKind::ItemEnum{values, ..} = krate.get(target_unraw_ty).kind {
        krate.extra_get(values).find_map(|field_id| {
          if let hir::Thing::NamedConst(name, cons) = *krate.get(field_id) {
            if name == sid {
              return Some(cons);
            }
          }
          None
        })
      } else {
        None
      }
    };

    if let Some(cons) = found_const {
      return hir::Expr {
        kind: hir::ExprKind::Const(cons),
        category: ExprCategory::RValue,
        pos,
        ety: target_unraw_ty,
      }.push_ok(ctx.cre);
    }

    let methods = if let Some(m) = ctx.type_impls.get(&target_ty) {
      Some(m)
    } else if let hir::TypeKind::GenericRaw { generic: g1, kind: k1 } = ctx.get(target_ty).kind {
      if let Some(m) = ctx.type_impls.get(&k1) {
        Some(m)
      } else {
        ctx.type_impls.iter().find_map(|(&impl_ty, m)| {
          if let hir::TypeKind::GenericRaw { generic: g2, kind: k2 } = ctx.get(impl_ty).kind {
            if g1 == g2 && k1 == k2 {
              return Some(m);
            }
          }
          None
        })
      }
    } else if let Some((_, base_ty, _)) = ctx.cmap.cache_generic_type_rev.get(&target_ty) {
      ctx.type_impls.get(base_ty)
    } else {
      ctx.type_impls.iter().find_map(|(&impl_ty, m)| {
        if let hir::TypeKind::GenericRaw { kind, .. } = ctx.get(impl_ty).kind {
          if kind == target_ty {
            return Some(m);
          }
        }
        None
      })
    };

    if let Some(methods) = methods {
      if let Some(items) = methods.methods.get(&sid) {
        let mut ordered_items: Vec<ast::ItemId> = items.clone();
        if methods.inherent_methods.contains(&sid) {
          ordered_items.sort_by_key(|&item| {
            if let ast::ItemKind::Impl { trait_ty: None, .. } = ctx.src.get(item).kind {
              0
            } else {
              1
            }
          });
        }

        for item in ordered_items {
          if let Success(Some(impl_id)) = ItemLow::low(ctx, sum, item) {
            let hir::ItemKind::Impl { methods, .. } = ctx.cre.get(impl_id).kind else { unreachable!() };
            let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue };
            let hir::ItemKind::Function { kind, .. } = ctx.cre.get(method).kind else { unreachable!() };

            return hir::Expr {
              kind: hir::ExprKind::GlobalRef(method),
              category: ExprCategory::RValue,
              ety: kind,
              pos,
            }.push_ok(ctx.cre);
          }
        }
      }
    }

    for &cid in ctx.imods {
      let dep_krate = ctx.ideps.get(cid);
      for impl_id in dep_krate.impl_items() {
        let hir::ItemKind::Impl { type_ty, methods, .. } = ctx.get(impl_id).kind else { continue; };
        if type_ty != target_ty && type_ty != target_unraw_ty {
          continue;
        }
        let Some(method) = ctx.extra_get(methods).find(|&m| ctx.get(m).name == Some(sid)) else { continue; };
        let hir::ItemKind::Function { kind, .. } = ctx.get(method).kind else { continue; };

        return hir::Expr {
          kind: hir::ExprKind::GlobalRef(method),
          category: ExprCategory::RValue,
          ety: kind,
          pos,
        }.push_ok(ctx.cre);
      }
    }

    ret_err!(sum, Message::error(HAS_NO_FIELD_NAMED_X.args(&[ctx.sin.str(sid)]), Label::new_pos(pos)))
  }

  pub fn lookup_method_with_trait(ctx: &mut Ctx, sum: &mut Summary, target_trait: hir::TypeId, current_id: hir::ExprId, rhs_id: hir::ExprId, sid: Sid, pos: impl Into<Span>) -> Status<(hir::ExprId, hir::TypeId)> {
    let this_ty = ctx.cre.get(current_id).ety;
    let rhs_ty = ctx.cre.get(rhs_id).ety;
    let pos_span = pos.into();

    let target_path = ctx.type_path(target_trait);

    if let Some(methods) = ctx.type_impls.get(&this_ty) {
      if let Some(items) = methods.methods.get(&sid) {
        for &item in items {
          if let Success(Some(impl_id)) = ItemLow::low(ctx, sum, item) {
            let hir::ItemKind::Impl { methods, trait_ty, .. } = ctx.cre.get(impl_id).kind else { continue; };

            let Some(trait_ty) = trait_ty else { continue; };

            let Some(mut impl_path) = ctx.type_path(trait_ty) else { continue; };
            while let hir::DefPath::Spec { base, .. } = *ctx.get(impl_path) {
              impl_path = base;
            }

            if Some(impl_path) != target_path {
              continue;
            }

            let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue; };
            let hir::ItemKind::Function { kind: fn_ty, .. } = ctx.cre.get(method).kind else { continue; };
            let hir::TypeKind::Fun { args, ret, .. } = ctx.get(fn_ty).kind else { continue; };

            if args.count() != 1 { continue; }
            let arg_thing = ctx.extra_get(args).next().unwrap();

            let hir::Thing::NamedType(_, param_ty) = *ctx.get(arg_thing) else { continue; };
            let param_unref_ty = match ctx.get(param_ty).kind {
              hir::TypeKind::Ref(inner, _) => inner,
              _ => param_ty,
            };
            let rhs_unref_ty = match ctx.get(rhs_ty).kind {
              hir::TypeKind::Ref(inner, _) => inner,
              _ => rhs_ty,
            };

            if param_ty != rhs_ty && param_unref_ty != rhs_unref_ty {
              continue;
            }

            return Success((
              hir::Expr {
                kind: hir::ExprKind::GlobalRef(method),
                category: ExprCategory::RValue,
                ety: fn_ty,
                pos: pos_span,
              }.push(ctx.cre),
              ret,
            ));
          }
        }
      }
    }

    for &cid in ctx.imods {
      let dep_krate = ctx.ideps.get(cid);
      for impl_id in dep_krate.impl_items() {
        let hir::ItemKind::Impl { type_ty, trait_ty, methods } = ctx.get(impl_id).kind else { continue; };
        if type_ty != this_ty {
          continue;
        }
        let Some(trait_ty) = trait_ty else { continue; };

        let Some(mut impl_path) = ctx.type_path(trait_ty) else { continue; };
        while let hir::DefPath::Spec { base, .. } = *ctx.get(impl_path) {
          impl_path = base;
        }

        if Some(impl_path) != target_path {
          continue;
        }

        let Some(method) = ctx.extra_get(methods).find(|&m| ctx.get(m).name == Some(sid)) else { continue; };
        let hir::ItemKind::Function { kind: fn_ty, .. } = ctx.get(method).kind else { continue; };
        let hir::TypeKind::Fun { args, ret, .. } = ctx.get(fn_ty).kind else { continue; };

        if args.count() != 1 { continue; }
        let arg_thing = ctx.extra_get(args).next().unwrap();

        let hir::Thing::NamedType(_, param_ty) = *ctx.get(arg_thing) else { continue; };
        let param_unref_ty = match ctx.get(param_ty).kind {
          hir::TypeKind::Ref(inner, _) => inner,
          _ => param_ty,
        };
        let rhs_unref_ty = match ctx.get(rhs_ty).kind {
          hir::TypeKind::Ref(inner, _) => inner,
          _ => rhs_ty,
        };

        if param_ty != rhs_ty && param_unref_ty != rhs_unref_ty {
          continue;
        }

        return Success((
          hir::Expr {
            kind: hir::ExprKind::GlobalRef(method),
            category: ExprCategory::RValue,
            ety: fn_ty,
            pos: pos_span,
          }.push(ctx.cre),
          ret,
        ));
      }
    }

    Fail
  }

  pub fn lookup_method_with_trait_unary(ctx: &mut Ctx, sum: &mut Summary, target_trait: hir::TypeId, current_id: hir::ExprId, sid: Sid, pos: impl Into<Span>) -> Status<(hir::ExprId, hir::TypeId)> {
    let this_ty = ctx.cre.get(current_id).ety;
    let pos_span = pos.into();

    let target_path = ctx.type_path(target_trait);

    if let Some(methods) = ctx.type_impls.get(&this_ty) {
      if let Some(items) = methods.methods.get(&sid) {
        for &item in items {
          if let Success(Some(impl_id)) = ItemLow::low(ctx, sum, item) {
            let hir::ItemKind::Impl { methods, trait_ty, .. } = ctx.cre.get(impl_id).kind else { continue; };

            let Some(trait_ty) = trait_ty else { continue; };

            let Some(mut impl_path) = ctx.type_path(trait_ty) else { continue; };
            while let hir::DefPath::Spec { base, .. } = *ctx.get(impl_path) {
              impl_path = base;
            }

            if Some(impl_path) != target_path {
              continue;
            }

            let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue; };
            let hir::ItemKind::Function { kind: fn_ty, .. } = ctx.cre.get(method).kind else { continue; };
            let hir::TypeKind::Fun { args, ret, .. } = ctx.get(fn_ty).kind else { continue; };

            if args.count() != 0 { continue; }

            return Success((
              hir::Expr {
                kind: hir::ExprKind::GlobalRef(method),
                category: ExprCategory::RValue,
                ety: fn_ty,
                pos: pos_span,
              }.push(ctx.cre),
              ret,
            ));
          }
        }
      }
    }

    for &cid in ctx.imods {
      let dep_krate = ctx.ideps.get(cid);
      for impl_id in dep_krate.impl_items() {
        let hir::ItemKind::Impl { type_ty, trait_ty, methods } = ctx.get(impl_id).kind else { continue; };
        if type_ty != this_ty {
          continue;
        }
        let Some(trait_ty) = trait_ty else { continue; };

        let Some(mut impl_path) = ctx.type_path(trait_ty) else { continue; };
        while let hir::DefPath::Spec { base, .. } = *ctx.get(impl_path) {
          impl_path = base;
        }

        if Some(impl_path) != target_path {
          continue;
        }

        let Some(method) = ctx.extra_get(methods).find(|&m| ctx.get(m).name == Some(sid)) else { continue; };
        let hir::ItemKind::Function { kind: fn_ty, .. } = ctx.get(method).kind else { continue; };
        let hir::TypeKind::Fun { args, ret, .. } = ctx.get(fn_ty).kind else { continue; };

        if args.count() != 0 { continue; }

        return Success((
          hir::Expr {
            kind: hir::ExprKind::GlobalRef(method),
            category: ExprCategory::RValue,
            ety: fn_ty,
            pos: pos_span,
          }.push(ctx.cre),
          ret,
        ));
      }
    }

    Fail
  }

}
