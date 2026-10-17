/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, Status::{self, Fail, Success}, StatusApi2, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_string_interner::Sid;

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
          if TypeMatch::matches_pos(ctx, sum, cand_self_ty, this_ty, pos_span).is_fail() {
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
          if TypeMatch::matches_pos(ctx, sum, cand_param_ty, user_ty, pos_span).is_fail() {
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
    let methods = ctx.type_impls.get(&this_ty).status()?;
    let items = methods.methods.get(&sid).status()?;

    for &item in items {
      if let Success(Some(impl_id)) = ItemLow::low(ctx, sum, item) {
        let hir::ItemKind::Impl { methods, .. } = ctx.cre.get(impl_id).kind else { unreachable!() };
        let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue };
        let hir::ItemKind::Function { kind, .. } = ctx.cre.get(method).kind else { unreachable!() };

        let callee = hir::Expr {
          kind: hir::ExprKind::GlobalRef(method),
          category: ExprCategory::RValue,
          ety: kind,
        }.push(ctx.cre);

        return hir::Expr {
          kind: hir::ExprKind::BoundSelfMethod {
            callee,
            this: current_id,
          },
          category: ExprCategory::RValue,
          ety: ctx.prims.ty_unit,
        }.push_ok(ctx.cre);
      }
    }

    hir::Expr {
      kind: hir::ExprKind::Error,
      category: ExprCategory::RValue,
      ety: ctx.prims.ty_error,
    }.push_ok(ctx.cre)
  }

  pub fn lookup_method_with_trait(ctx: &mut Ctx, sum: &mut Summary, target_trait: hir::TypeId, current_id: hir::ExprId, rhs_id: hir::ExprId, sid: Sid, pos: impl Into<Span>) -> Status<(hir::ExprId, hir::TypeId)> {
    let this_ty = ctx.cre.get(current_id).ety;
    let rhs_ty = ctx.cre.get(rhs_id).ety;
    let pos_span = pos.into();

    let methods = ctx.type_impls.get(&this_ty).status()?;
    let items = methods.methods.get(&sid).status()?;

    let target_path = ctx.get(target_trait).path;

    for &item in items {
      if let Success(Some(impl_id)) = ItemLow::low(ctx, sum, item) {
        let hir::ItemKind::Impl { methods, trait_ty, .. } = ctx.cre.get(impl_id).kind else { continue; };

        let Some(trait_ty) = trait_ty else { continue; };

        let mut impl_path = ctx.get(trait_ty).path;
        while let hir::DefPath::Spec { base, .. } = *ctx.get(impl_path) {
          impl_path = base;
        }

        if impl_path != target_path {
          continue;
        }

        let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue; };
        let hir::ItemKind::Function { kind: fn_ty, .. } = ctx.cre.get(method).kind else { continue; };
        let hir::TypeKind::Fun { args, ret, .. } = ctx.get(fn_ty).kind else { continue; };

        if args.count() != 1 { continue; }
        let arg_thing = ctx.extra_get(args).next().unwrap();

        let hir::Thing::NamedType(_, param_ty) = *ctx.get(arg_thing) else { continue; };
        if TypeMatch::matches_pos(ctx, sum, param_ty, rhs_ty, pos_span).is_fail() {
          continue;
        }

        return Success((
          hir::Expr {
            kind: hir::ExprKind::GlobalRef(method),
            category: ExprCategory::RValue,
            ety: fn_ty,
          }.push(ctx.cre),
          ret,
        ));
      }
    }

    Fail
  }

}
