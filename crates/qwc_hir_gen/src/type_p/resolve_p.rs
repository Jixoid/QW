/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Span, Status, Summary, msg::*, ret_err};
use qwc_resolve::Resolver;
use qwc_ast::{self as ast, Ident};
use qwc_hir::{self as hir, PushOkApi, TypeAttr};
use qwc_resolve as resolve;

use crate::{Ctx, ctx, ItemLow, TypeLow};



// Resolve
pub fn low_nick(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Type, ident: Ident) -> Status<hir::TypeId> {
  let (kind, lscp, _) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).lookup(sum, ident)?.get_k();
  low_resolved(ctx, sum, it, kind, lscp)
}

pub fn low_path(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Type, rng: ast::TypeRng) -> Status<hir::TypeId> {
  let mut segment = vec![];

  for id in ctx.src.extra_get(rng) {
    let it = ctx.src.get(id);

    if let ast::TypeKind::Nick(ident) = it.kind { segment.push(ident) } else { unreachable!() }
  }

  let (kind, lscp, _) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(sum, &segment)?.get_k();
  low_resolved(ctx, sum, it, kind, lscp)
}


fn low_resolved(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Type, kind: resolve::ScopeKind, lscp: &resolve::Scope) -> Status<hir::TypeId> {
  low_resolved_kind(ctx, sum, it.pos.into(), kind, lscp)
}

pub fn low_resolved_kind(ctx: &mut Ctx, sum: &mut Summary, pos: Span, kind: resolve::ScopeKind, lscp: &resolve::Scope) -> Status<hir::TypeId> {
  let ty = match kind {
    resolve::ScopeKind::Ast(ast_kind) => match ast_kind {
      resolve::ScopeKindAst::Type(item) => {
        let item = ItemLow::low(ctx!(lscp -> ctx), sum, item)?.unwrap();
        let hir::ItemKind::Using { kind } = ctx.get(item).kind else { unreachable!() };
        kind
      }

      resolve::ScopeKindAst::TypeParam(generic, idx, thing) => {
        let name = match ctx.src.get(thing).kind {
          ast::ThingKind::Name(name) => name,
          ast::ThingKind::NamedType(name, _) => name,
          _ => unreachable!(),
        };

        let generic_hir = ItemLow::low(ctx, sum, generic)?.unwrap();
        let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name: name.sid() });
        ctx.tin.ty_generic_type(ctx.cre, path, generic_hir, idx)
      }

      resolve::ScopeKindAst::AssocType(idx, field) => {
        let field_obj = ctx.src.get(field);
        match field_obj.kind {
          ast::FieldKind::Type(Some(ty)) => {
            TypeLow::low(ctx!(lscp -> ctx), sum, ty)?
          }

          ast::FieldKind::Type(None) => {
            let trait_ty = *ctx.cmap.current_trait.last().expect("associated type outside trait");
            let name = field_obj.name.unwrap().sid();
            let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });
            hir::Type {
              kind: hir::TypeKind::AssociatedType { trait_ty, idx },
              layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
              attr: TypeAttr::empty(),
              path,
            }.push(ctx.cre)
          }

          _ => unreachable!(),
        }
      }

      // Expr
      resolve::ScopeKindAst::Expr(item) => {
        let span = ctx.src.get(item).pos;

        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
      }

      // Module
      resolve::ScopeKindAst::Module(item) => {
        let span = ctx.src.get(item).pos;

        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "module"]), Label::new_pos(span)))
      }

      resolve::ScopeKindAst::ExprParam(.., thing) => {
        let span = ctx.src.get(thing).pos;

        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(span)))
      }
      
      resolve::ScopeKindAst::GenericType(generic, kind) => {
        let generic_item = ItemLow::low(ctx, sum, generic)?.unwrap();
        let path = ctx.get(generic_item).path;
        let kind = match ctx.cmap.cache_type.get(&kind) {
          Some(&cached) => cached,
          None => TypeLow::low(ctx, sum, kind)?,
        };
        hir::Type {
          kind: hir::TypeKind::GenericRaw { generic: generic_item, kind },
          attr: TypeAttr::empty(),
          layout: hir::Layout::new_meta(hir::LayoutBy::QW),
          path,
        }.push(ctx.cre)
      }

      resolve::ScopeKindAst::GenericExpr(..) => {
        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(pos)))
      }

      resolve::ScopeKindAst::Local(..) => panic!("value in type position"),
    },

    resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
      resolve::ScopeKindHir::Type(ty) => ty,

      resolve::ScopeKindHir::Expr(..) | resolve::ScopeKindHir::Module(..) => {
        ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(pos)))
      }
    }
  };

  Status::Success(ty)
}
