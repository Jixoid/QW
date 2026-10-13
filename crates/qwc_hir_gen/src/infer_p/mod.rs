/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_string_interner::Sid;

use crate::{Ctx, ItemLow, TypeMatch};


pub struct Infer;

impl Infer {

  pub fn resolve_method(ctx: &mut Ctx, this: hir::ExprId, callee: hir::ExprId, user_args: &[hir::ExprId], pos: impl Into<Span>) -> Result<hir::ExprId, Message> {
    let this_ty = ctx.cre.get(this).ety;

    let method_name = if let hir::ExprKind::GlobalRef(item) = ctx.cre.get(callee).kind {
      ctx.cre.get(item).name
    } else {
      None
    };

    let Some(method_name) = method_name else { return Ok(callee); };
    let Some(type_methods) = ctx.type_impls.get(&this_ty) else { return Ok(callee); };
    let Some(cand_impl_items) = type_methods.methods.get(&method_name) else { return Ok(callee); };

    if cand_impl_items.len() <= 1 {
      return Ok(callee);
    }

    let pos_span = pos.into();
    let mut matched = vec![];

    for &impl_ast_id in cand_impl_items {
      if let Ok(Some(impl_hir_id)) = ItemLow::low(ctx, impl_ast_id) {
        let hir::ItemKind::Impl { methods, .. } = ctx.cre.get(impl_hir_id).kind else { continue; };
        let Some(cand_method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name == Some(method_name)) else { continue; };
        let hir::ItemKind::Function { kind: cand_fn_ty, .. } = ctx.cre.get(cand_method).kind else { continue; };
        let hir::TypeKind::Fun { self_kind: cand_self, args: cand_params, .. } = ctx.get(cand_fn_ty).kind else { continue; };

        if let Some(cand_self_ty) = cand_self {
          if TypeMatch::matches_pos(ctx, cand_self_ty, this_ty, pos_span).is_err() {
            continue;
          }
        }

        if cand_params.count() as usize != user_args.len() {
          continue;
        }

        let mut all_match = true;
        for (&user_arg, cand_param) in user_args.iter().zip(ctx.extra_get(cand_params)) {
          let hir::Thing::NamedType(_, cand_param_ty) = *ctx.get(cand_param) else { panic!() };
          let user_ty = ctx.cre.get(user_arg).ety;
          if TypeMatch::matches_pos(ctx, cand_param_ty, user_ty, pos_span).is_err() {
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
      Ok(hir::Expr {
        kind: hir::ExprKind::GlobalRef(matched_method),
        category: ExprCategory::RValue,
        ety: matched_fn_ty,
      }.push(ctx.cre))
    } else if matched.len() > 1 {
      Err(Message::error(AMBIGUOUS_METHOD_CALL, Label::new_pos(pos_span)))
    } else {
      Ok(callee)
    }
  }

  /// Looks up a method by name on the receiver's type, returning a `BoundSelfMethod` expression.
  pub fn lookup_method(ctx: &mut Ctx, current_id: hir::ExprId, sid: Sid) -> Option<hir::ExprId> {
    let this_ty = ctx.cre.get(current_id).ety;
    let methods = ctx.type_impls.get(&this_ty)?;
    let items = methods.methods.get(&sid)?;

    for &item in items {
      if let Ok(Some(impl_id)) = ItemLow::low(ctx, item) {
        let hir::ItemKind::Impl { methods, .. } = ctx.cre.get(impl_id).kind else { unreachable!() };
        let Some(method) = ctx.cre.extra_get(methods).find(|&m| ctx.cre.get(m).name.unwrap() == sid) else { continue };
        let hir::ItemKind::Function { kind, .. } = ctx.cre.get(method).kind else { unreachable!() };

        let callee = hir::Expr {
          kind: hir::ExprKind::GlobalRef(method),
          category: ExprCategory::RValue,
          ety: kind,
        }.push(ctx.cre);

        return Some(hir::Expr {
          kind: hir::ExprKind::BoundSelfMethod {
            callee,
            this: current_id,
          },
          category: ExprCategory::RValue,
          ety: ctx.prims.ty_unit,
        }.push(ctx.cre));
      }
    }

    Some(hir::Expr {
      kind: hir::ExprKind::Error,
      category: ExprCategory::RValue,
      ety: ctx.prims.ty_error,
    }.push(ctx.cre))
  }

  pub fn lookup_method_with_trait(ctx: &mut Ctx, target_trait: hir::TypeId, current_id: hir::ExprId, rhs_id: hir::ExprId, sid: Sid, pos: impl Into<Span>) -> Option<(hir::ExprId, hir::TypeId)> {
    let this_ty = ctx.cre.get(current_id).ety;
    let rhs_ty = ctx.cre.get(rhs_id).ety;
    let pos_span = pos.into();

    let methods = ctx.type_impls.get(&this_ty)?;
    let items = methods.methods.get(&sid)?;

    let target_path = ctx.get(target_trait).path;

    for &item in items {
      if let Ok(Some(impl_id)) = ItemLow::low(ctx, item) {
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
        if TypeMatch::matches_pos(ctx, param_ty, rhs_ty, pos_span).is_err() {
          continue;
        }

        return Some((
          hir::Expr {
            kind: hir::ExprKind::GlobalRef(method),
            category: ExprCategory::RValue,
            ety: fn_ty,
          }.push(ctx.cre),
          ret,
        ));
      }
    }

    None
  }

}
