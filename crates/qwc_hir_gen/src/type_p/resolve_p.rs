/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_resolve::Resolver;
use qwc_ast::{self as ast, Ident};
use qwc_hir::{self as hir, PushOkApi, TypeAttr};
use qwc_resolve as resolve;

use crate::{Ctx, ctx, ItemLow, TypeLow};



// Resolve
pub fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::TypeId, Message> {
  let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).lookup(ident)?.get_k();
  low_resolved(ctx, kind, lscp, span)
}

pub fn low_path(ctx: &mut Ctx, rng: ast::TypeRng) -> Result<hir::TypeId, Message> {
  let mut segment = vec![];

  for id in ctx.src.extra_get(rng) {
    let it = ctx.src.get(id);

    if let ast::TypeKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
  }

  let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(&segment)?.get_k();
  low_resolved(ctx, kind, lscp, span)
}


fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope, span: Span) -> Result<hir::TypeId, Message> {
  let ty = match kind {
    resolve::ScopeKind::Ast(ast_kind) => match ast_kind {
      resolve::ScopeKindAst::Type(ty) => TypeLow::low(ctx!(lscp -> ctx), ty)?,

      resolve::ScopeKindAst::TypeParam(idx, thing) => {
        match ctx.src.get(thing).kind {
          ast::ThingKind::NamedType(_, ty) => TypeLow::low(ctx!(lscp -> ctx), ty)?,

          ast::ThingKind::Name(_) => ctx.tin.ty_generic_type(ctx.cre, idx),

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

      resolve::ScopeKindAst::ExprParam(_, thing) => {
        let span = ctx.src.get(thing).pos;

        return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
      }
      
      resolve::ScopeKindAst::GenericType(generic, kind) => {
        hir::Type {
          kind: hir::TypeKind::GenericRaw { generic: ItemLow::low(ctx, generic)?.unwrap(), kind: TypeLow::low(ctx, kind)? },
          attr: TypeAttr::empty(),
          layout: hir::Layout::new_meta(hir::LayoutBy::QW),
        }.push(ctx.cre)
      }

      resolve::ScopeKindAst::GenericExpr(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
      }

      resolve::ScopeKindAst::Local(..) => panic!("value in type position"),
    },

    resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
      resolve::ScopeKindHir::Type(ty) => ty,

      resolve::ScopeKindHir::Expr(..) | resolve::ScopeKindHir::Module(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
      }
    }
  };

  Ok(ty)
}
