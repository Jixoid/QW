use qwc_ast::{self as ast, Ident};
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_hir::{self as hir, ExprCategory};
use qwc_resolve::{self as resolve, Resolver};

use crate::{ItemLow, TypeLow, ctx, hgen::Ctx};


// Resolve
pub fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::ExprId, Message> {
  let (kind, lscp, span) = Resolver::with_locals(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods, Some(ctx.loc)).lookup(ident)?.get_k();
  low_resolved(ctx, kind, lscp, span)
}

pub fn low_path(ctx: &mut Ctx, rng: ast::ExprRng) -> Result<hir::ExprId, Message> {
  let mut segment = vec![];

  for id in ctx.src.extra_get(rng) {
    let it: &ast::Expr = ctx.src.get(id);

    if let ast::ExprKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
  }

  let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(&segment)?.get_k();
  low_resolved(ctx, kind, lscp, span)
}

pub fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope, span: Span) -> Result<hir::ExprId, Message> {
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
        let it: &hir::Item = ctx.cre.get(id);

        let kind = it.symbol_kind().unwrap();
        
        let ism = it.assignable().unwrap();


        // Post
        let this = hir::Expr{
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::lvalue(ism), // TODO!
          ety: kind,
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindAst::Type(..) | resolve::ScopeKindAst::TypeParam(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
      }

      resolve::ScopeKindAst::Module(id) => {
        let span = (ctx.src.get(id) as &ast::Item).pos;
        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
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
    },

    resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
      resolve::ScopeKindHir::Expr(id, ty) => {
        let ty = ctx.low_hir_type(ty);
        let kind = ctx.tin.ty_ref(ctx.cre, ty, false); // TODO!

        let this = hir::Expr {
          kind: hir::ExprKind::GlobalRef(id),
          category: ExprCategory::RValue,
          ety: kind,
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKindHir::Type(..) | resolve::ScopeKindHir::Module(..) => {
        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
      }
    },
  };

  Ok(expr)
}
