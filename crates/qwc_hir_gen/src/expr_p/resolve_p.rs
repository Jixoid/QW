/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{self as ast, Ident};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, ExprCategory, PushOkApi};
use qwc_resolve::{self as resolve, Resolver};

use crate::{ItemLow, TypeLow, ctx, hgen::Ctx};


// Resolve
pub fn low_nick(ctx: &mut Ctx, pos: Span, ident: Ident) -> Result<hir::ExprId, Message> {
  let (kind, lscp, _) = Resolver::with_locals(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods, Some(ctx.loc)).lookup(ident)?.get_k();
  low_resolved(ctx, kind, lscp, pos)
}

pub fn low_self(ctx: &mut Ctx, pos: Span) -> Result<hir::ExprId, Message> {
  let self_sid = ctx.sin.sid_self();

  let Some(id) = ctx.loc.lookup(&self_sid) else {
    return Err(Message::error(CANNOT_FIND_X_IN_SCOPE.args(&["self"]), Label::new_pos(pos)))
  };

  
  // Post
  let local = ctx.loc.get_local(id);

  let this = hir::Expr {
    kind: hir::ExprKind::LocalRef(id),
    category: ExprCategory::lvalue(local.ism),
    ety: local.ty,
  };

  Ok(ctx.cre.push(this))
}

pub fn low_self_big(ctx: &mut Ctx, pos: Span) -> Result<hir::ExprId, Message> {
  use qwc_diagnostic::msg::SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT;
  let kind = match ctx.cmap.self_ty.last() {
    Some(&v) => v,
    None => return Err(Message::error(SELF_TYPE_IS_ONLY_ALLOWED_IN_ASSOCIATED_CONTEXT, Label::new_pos(pos)))
  };

  let this = hir::Expr {
    kind: hir::ExprKind::TypeOf { kind },
    category: ExprCategory::RValue,
    ety: ctx.tin.ty_meta(ctx.cre, kind),
  };

  Ok(ctx.cre.push(this))
}

pub fn low_path(ctx: &mut Ctx, pos: Span, rng: ast::ExprRng) -> Result<hir::ExprId, Message> {
  let mut segment = vec![];

  for id in ctx.src.extra_get(rng) {
    let it = ctx.src.get(id);

    if let ast::ExprKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
  }

  let (kind, lscp, _) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(&segment)?.get_k();
  low_resolved(ctx, kind, lscp, pos)
}

pub fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope, pos: Span) -> Result<hir::ExprId, Message> {
  let expr = match kind {
    resolve::ScopeKind::Ast(ast_kind) => match ast_kind {
      resolve::ScopeKindAst::ExprParam(thing) => {
        if let ast::Thing::NamedType(_, ty) = ctx.src.get(thing) as &ast::Thing {
          let ty = TypeLow::low(ctx!(lscp -> ctx), *ty)?;

          // Post
          let this = hir::Expr{
            kind: hir::ExprKind::GenericExpr,
            category: ExprCategory::RValue,
            ety: ty,
          };

          ctx.cre.push(this)
        }
        else { panic!() }
      }

      resolve::ScopeKindAst::Expr(id) => {
        let id = ItemLow::low(ctx!(lscp -> ctx), id)?.unwrap();
        let it = ctx.cre.get(id);

        let kind = it.symbol_kind().unwrap();
        
        let ism = it.assignable().unwrap();


        // Post
        let this = hir::Expr{
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::lvalue(ism),
          ety: kind,
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindAst::Type(kind) => {
        let kind = TypeLow::low(ctx, kind)?;

        // Post
        let this = hir::Expr {
          kind: hir::ExprKind::TypeOf { kind },
          category: ExprCategory::RValue,
          ety: ctx.tin.ty_meta(ctx.cre, kind),
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindAst::TypeParam(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(pos)));
      }

      resolve::ScopeKindAst::Module(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND.args(&["expr", "module"]), Label::new_pos(pos)));
      }

      resolve::ScopeKindAst::Local(id) => {
        let local = ctx.loc.get_local(id);
        let this = hir::Expr {
          kind: hir::ExprKind::LocalRef(id),
          category: ExprCategory::lvalue(local.ism),
          ety: local.ty,
        };
        ctx.cre.push(this)
      }
    }

    resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
      resolve::ScopeKindHir::Expr(id, ty) => {
        let kind = ctx.tin.ty_ref(ctx.cre, ty, false); // TODO!

        hir::Expr {
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::RValue,
          ety: kind,
        }.push(ctx.cre)
      }

      resolve::ScopeKindHir::Type(..) | resolve::ScopeKindHir::Module(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(pos)));
      }
    }
  };

  Ok(expr)
}
