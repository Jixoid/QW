/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{self as ast, Ident};
use qwc_diagnostic::{Label, Message, Status::{self, Success}, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, ExprCategory, PushOkApi, TypeAttr};
use qwc_resolve::{self as resolve, Resolver};

use crate::{ItemLow, TypeLow, ctx, Ctx};


// Resolve
pub fn low_nick(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, ident: Ident) -> Status<hir::ExprId> {
  let (kind, lscp, _) = Resolver::with_locals(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods, Some(ctx.loc)).lookup(sum, ident)?.get_k();
  low_resolved(ctx, sum, it, kind, lscp)
}

pub fn low_self(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr) -> Status<hir::ExprId> {
  let self_sid = ctx.sin.sid_self();

  let Some(id) = ctx.loc.lookup(&self_sid) else {
    ret_err!(sum, Message::error(CANNOT_FIND_X_IN_SCOPE.args(&["self"]), Label::new_pos(it.pos)))
  };

  
  // Post
  let local = ctx.loc.get_local(id);

  let this = hir::Expr {
    kind: hir::ExprKind::LocalRef(id),
    category: ExprCategory::lvalue(local.ism),
    pos: it.pos,
    ety: local.ty,
  };

  Success(ctx.cre.push(this))
}

pub fn low_self_big(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr,) -> Status<hir::ExprId> {
  use qwc_diagnostic::msg::SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT;
  let kind = match ctx.cmap.self_ty.last() {
    Some(&v) => v,
    None => ret_err!(sum, Message::error(SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT, Label::new_pos(it.pos)))
  };

  let this = hir::Expr {
    kind: hir::ExprKind::TypeOf { kind },
    category: ExprCategory::RValue,
    pos: it.pos,
    ety: ctx.tin.ty_meta(ctx.cre, kind),
  };

  Success(ctx.cre.push(this))
}

pub fn low_path(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, rng: ast::ExprRng) -> Status<hir::ExprId> {
  let ids = ctx.src.extra_get(rng).collect::<Vec<_>>();
  if ids.is_empty() {
    unreachable!();
  }

  // First element is SelfB() -> e.g. Self::new
  if let ast::ExprKind::SelfB() = ctx.src.get(ids[0]).kind {
    let self_ty = match ctx.cmap.self_ty.last() {
      Some(&v) => v,
      None => ret_err!(sum, Message::error(SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT, Label::new_pos(it.pos))),
    };

    if ids.len() < 2 {
      let this = hir::Expr {
        kind: hir::ExprKind::TypeOf { kind: self_ty },
        category: ExprCategory::RValue,
        pos: it.pos,
        ety: ctx.tin.ty_meta(ctx.cre, self_ty),
      };
      return Success(ctx.cre.push(this));
    }

    let ast::ExprKind::Nick(method_ident) = ctx.src.get(ids[1]).kind else {
      ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(it.pos)));
    };

    return crate::infer_p::Infer::lookup_static_method(ctx, sum, self_ty, method_ident.sid(), it.pos);
  }

  // Extract identifiers
  let mut segment = vec![];
  for &id in &ids {
    let item = ctx.src.get(id);
    if let ast::ExprKind::Nick(ident) = item.kind {
      segment.push(ident);
    } else {
      ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(item.pos)));
    }
  }

  // Check if prefix resolves to a type (e.g. Vec2::new or math::Vec2::new)
  if segment.len() >= 2 {
    let prefix = &segment[..segment.len() - 1];
    let method_ident = segment[segment.len() - 1];

    let mut test_sum = Summary::new();
    if let Success(target) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(&mut test_sum, prefix) {
      let (kind, lscp, _) = target.get_k();
      let is_type = matches!(
        kind,
        resolve::ScopeKind::Ast(resolve::ScopeKindAst::Type(..) | resolve::ScopeKindAst::GenericType(..) | resolve::ScopeKindAst::TypeParam(..))
          | resolve::ScopeKind::Hir(resolve::ScopeKindHir::Type(..))
      );

      if is_type {
        let target_ty = crate::type_p::resolve_p::low_resolved_kind(ctx, sum, it.pos, kind, lscp)?;
        return crate::infer_p::Infer::lookup_static_method(ctx, sum, target_ty, method_ident.sid(), it.pos);
      }
    }
  }

  let (kind, lscp, _) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(sum, &segment)?.get_k();
  low_resolved(ctx, sum, it, kind, lscp)
}


pub fn low_resolved(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Expr, kind: resolve::ScopeKind, lscp: &resolve::Scope) -> Status<hir::ExprId> {
  let expr = match kind {
    resolve::ScopeKind::Ast(ast_kind) => match ast_kind {
      resolve::ScopeKindAst::ExprParam(generic, idx, thing) => {
        let ast::ThingKind::NamedType(_, ty) = ctx.src.get(thing).kind else { unreachable!() };
        
        let ty = TypeLow::low(ctx!(lscp -> ctx), sum, ty)?;
        let generic_hir = ItemLow::low(ctx, sum, generic)?.unwrap();

        // Post
        let this = hir::Expr {
          kind: hir::ExprKind::GenericExpr { generic: generic_hir, idx },
          category: ExprCategory::RValue,
          pos: it.pos,
          ety: ty,
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindAst::Expr(id) => {
        let id = ItemLow::low(ctx!(lscp -> ctx), sum, id)?.unwrap();
        let item = ctx.cre.get(id);

        let kind = item.symbol_kind().unwrap();
        
        let ism = item.assignable().unwrap();


        // Post
        hir::Expr {
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::lvalue(ism),
          pos: it.pos,
          ety: kind,
        }.push(ctx.cre)
      }

      resolve::ScopeKindAst::Type(item) => {
        let item = ItemLow::low(ctx!(lscp -> ctx), sum, item)?.unwrap();
        let hir::ItemKind::Using { kind } = ctx.get(item).kind else { unreachable!() };

        // Post
        let this = hir::Expr {
          kind: hir::ExprKind::TypeOf { kind },
          category: ExprCategory::RValue,
          pos: it.pos,
          ety: ctx.tin.ty_meta(ctx.cre, kind),
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindAst::TypeParam(..) | resolve::ScopeKindAst::AssocType(..) => {
        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(it.pos)));
      }

      resolve::ScopeKindAst::Module(..) => {
        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["expr", "module"]), Label::new_pos(it.pos)));
      }

      resolve::ScopeKindAst::GenericType(generic, kind) => {
        let generic_item = ItemLow::low(ctx, sum, generic)?.unwrap();
        let kind = match ctx.cmap.cache_type.get(&kind) {
          Some(&cached) => cached,
          None => TypeLow::low(ctx, sum, kind)?,
        };
        let kind = hir::Type {
          kind: hir::TypeKind::GenericRaw { generic: generic_item, kind },
          attr: TypeAttr::empty(),
          layout: hir::Layout::META,
        }.push(ctx.cre);

        let this = hir::Expr {
          kind: hir::ExprKind::TypeOf { kind },
          category: ExprCategory::RValue,
          pos: it.pos,
          ety: ctx.tin.ty_meta(ctx.cre, kind),
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindAst::GenericExpr(generic, expr) => {
        let generic = ItemLow::low(ctx, sum, generic)?.unwrap();
        let id = ItemLow::low(ctx!(lscp -> ctx), sum, expr)?.unwrap();
        let item = ctx.cre.get(id);

        let kind = item.symbol_kind().unwrap();
        
        let ism = item.assignable().unwrap();

        let expr = hir::Expr{
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::lvalue(ism),
          pos: it.pos,
          ety: kind,
        }.push(ctx.cre);

        let kind = hir::Type {
          kind: hir::TypeKind::GenericRaw { generic, kind },
          attr: TypeAttr::empty(),
          layout: hir::Layout::META,
        }.push(ctx.cre);
        
        
        // Post
        hir::Expr {
          kind: hir::ExprKind::GenericRaw { generic, kind: expr },
          category: ExprCategory::RValue,
          pos: it.pos,
          ety: kind,
        }.push(ctx.cre)
      }

      resolve::ScopeKindAst::Local(id) => {
        let local = ctx.loc.get_local(id);
        let this = hir::Expr {
          kind: hir::ExprKind::LocalRef(id),
          category: ExprCategory::lvalue(local.ism),
          pos: it.pos,
          ety: local.ty,
        };
        ctx.cre.push(this)
      }
    }

    resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
      resolve::ScopeKindHir::Expr(id, ty) => {
        let kind = if let hir::TypeKind::Fun { .. } = ctx.get(ty).kind {
          ty
        } else {
          ctx.tin.ty_ref(ctx.cre, ty, false)
        };

        hir::Expr {
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::RValue,
          pos: it.pos,
          ety: kind,
        }.push(ctx.cre)
      }

      resolve::ScopeKindHir::Type(kind) => {
        let this = hir::Expr {
          kind: hir::ExprKind::TypeOf { kind },
          category: ExprCategory::RValue,
          ety: ctx.tin.ty_meta(ctx.cre, kind),
          pos: it.pos,
        };

        ctx.cre.push(this)
      }

      qwc_resolve::ScopeKindHir::GenericType(generic, kind) => {
        let kind = hir::Type {
          kind: hir::TypeKind::GenericRaw { generic, kind },
          attr: TypeAttr::empty(),
          layout: hir::Layout::META,
        }.push(ctx.cre);

        let this = hir::Expr {
          kind: hir::ExprKind::TypeOf { kind },
          category: ExprCategory::RValue,
          pos: it.pos,
          ety: ctx.tin.ty_meta(ctx.cre, kind),
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindHir::Module(..) => {
        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["expr", "module"]), Label::new_pos(it.pos)));
      }
    }
  };

  Success(expr)
}
