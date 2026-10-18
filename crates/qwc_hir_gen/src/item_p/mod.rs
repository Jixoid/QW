/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, Status, StatusApi, Summary, msg::*, ret_err};
use qwc_ast::{self as ast, Attribute};
use qwc_hir::{self as hir, ItemAttr, PushOkApi};

use crate::{Ctx, ExprLow, TypeLow, TypeMatch, ctx, FunCtx};

mod impl_p;
mod ns_p;



pub struct ItemLow;

impl ItemLow {

  pub fn low(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId) -> Status<Option<hir::ItemId>> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Status::Success(id) }

    let it = ctx.src.get(id);

    let it = match it.kind {
      // Scope
      ast::ItemKind::Krate(rng) => Some(ns_p::low_krate(ctx, sum, id, it, rng)?),

      ast::ItemKind::Module(rng) | ast::ItemKind::ModuleFile(rng, ..) => Some(ns_p::low_module(ctx, sum, id, it, rng)?),
      
      ast::ItemKind::Generic{ctn: rng, params: args, ..} => Some(ns_p::low_generic(ctx, sum, id, it, rng, args)?),


      // Symbols
      ast::ItemKind::Fun{kind, blok} => Some(Self::low_fun(ctx, sum, id, it, kind, blok)?),
      
      ast::ItemKind::Task{kind, blok} => Some(Self::low_task(ctx, sum, id, it, kind, blok)?),
      
      ast::ItemKind::Let{kind, value, ism} => Some(Self::low_let(ctx, sum, id, it, kind, value, ism)?),
      
      ast::ItemKind::Impl{type_ty, trait_ty, ctn} => Some(impl_p::low_impl(ctx, sum, id, it, type_ty, trait_ty, ctn)?),


      // Using
      ast::ItemKind::Using(kind)  => Some(Self::low_using(ctx, sum, id, it, kind)?),
      ast::ItemKind::ItemTy(kind) => Some(Self::low_itemty(ctx, sum, id, it, kind)?),


      // Unexpected
      ast::ItemKind::ModuleUnloaded => panic!("ast object that should not be present"),
      
      // Impl
      ast::ItemKind::Import(..) => None,
    };

    ctx.cmap.cache_item.insert(id, it);

    Status::Success(it)
  }


  fn low_using(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Status<hir::ItemId> {
    let kind = TypeLow::low(ctx, sum, kind)?;

    let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;

    let name = it.name.unwrap().sid();
    let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });

    // Post
    hir::Item {
      name: Some(name),
      kind: hir::ItemKind::Using { kind },
      path,
      vis, attr
    }.push_ok(ctx.cre)
  }

  fn low_itemty(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Status<hir::ItemId> {
    let name = it.name.unwrap().sid();
    let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });
    
    let lscp = ctx.scp.get(&id.to_any()).unwrap_or(ctx.lscp);
    let kind = TypeLow::low(ctx!(lscp, path -> ctx), sum, kind)?;
    
    let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;
    
    // Post
    hir::Item {
      name: Some(name),
      kind: hir::ItemKind::Using { kind },
      path,
      vis, attr
    }.push_ok(ctx.cre)
  }


  fn low_let(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, kind: Option<ast::TypeId>, expr: ast::ExprId, ism: bool) -> Status<hir::ItemId> {
    let name = it.name.unwrap().sid();
    let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });

    let kind = kind.map(|id| TypeLow::low(ctx, sum, id)).transpose()?;

    let fctx = FunCtx::new_const(ctx.src.get(expr).pos);
    let expr = ExprLow::low(ctx, &fctx, sum, expr)?;

    let (hir::ExprKind::Const(..) | hir::ExprKind::GlobalRef(..)) = ctx.cre.get(expr).kind else {
      ret_err!(sum, Message::error(GLOBAL_VARIABLES_CAN_ONLY_BE_CONSTANT, Label::new_pos(it.pos)))
    };

    let expr_ty = ctx.cre.get(expr).ety;


    let kind = if let Some(kind) = kind {
      TypeMatch::matches(ctx, sum, kind, expr_ty, it.pos, None)?;
      kind
    } else {
      expr_ty
    };


    let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      name: Some(name),
        kind: hir::ItemKind::Variable { kind, expr, ism },
      path,
      vis, attr
    }.push_ok(ctx.cre)
  }


  fn low_fun(ctx: &mut Ctx, sum: &mut Summary, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Status<hir::ItemId> {
    let name = it.name.unwrap().sid();
    let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });


    let hir_kind = TypeLow::low(ctx!(path path -> ctx), sum, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ret} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { unreachable!() };
      let ast::ThingKind::NamedType(name_pos, ..) = ctx.src.get(id_pos).kind else { unreachable!() };

      loc.insert(name, kind, false, name_pos);
    }

    let fctx = FunCtx::new_fun(ret, ctx.src.get(kind).pos);

    let expr_hir = ExprLow::low(ctx!(loc loc -> ctx), &fctx, sum, expr.unwrap())?;
    
    let expr_ty = ctx.cre.get(expr_hir).ety;

    // Trait Patch
    let ret = if let hir::TypeKind::Trait{..} = ctx.get(ret).kind && let hir::TypeKind::TraitFrom{..} = ctx.get(expr_ty).kind {
      let fun_sign = ctx.cre.get_mut(hir_kind);

      if let hir::TypeKind::Fun { self_kind, args, .. } = fun_sign.kind {
        fun_sign.kind = hir::TypeKind::Fun { self_kind, args, ret: expr_ty };
      }

      expr_ty
    } else { ret };

    TypeMatch::matches(ctx, sum, ret, expr_ty, it.pos, None)?;

    let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      name: Some(name),
      kind: hir::ItemKind::Function {
        expr: expr_hir,
        kind: hir_kind,
      },
      path,
      vis, attr
    }.push_ok(ctx.cre)
  }

  fn low_task(_ctx: &mut Ctx, _sum: &mut Summary, _id: ast::ItemId, _it: &ast::Item, _kind: ast::TypeId, _expr: Option<ast::ExprId>) -> Status<hir::ItemId> {
    todo!()
  }

  pub(crate) fn prep_fun_field(ctx: &mut Ctx, sum: &mut Summary, id: ast::FieldId, it: &ast::Field, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Status<(hir::ItemId, hir::TypeId, qwc_resolve::LocalScopeManager, FunCtx)> {
    if expr.is_none() {
      ret_err!(sum, Message::error(ASSOCIATED_FUNCTION_IN_X_WITHOUT_BODY.args(&["impl"]), Label::new_pos(it.pos)))
    }

    let name = it.name.unwrap().sid();
    let path = ctx.cre.push(hir::DefPath::Path { base: ctx.path, name });

    let hir_kind = TypeLow::low(ctx!(path path -> ctx), sum, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ret} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { unreachable!() };
      let ast::ThingKind::NamedType(name_pos, ..) = ctx.src.get(id_pos).kind else { unreachable!() };

      loc.insert(name, kind, false, name_pos);
    }

    let fctx = FunCtx::new_fun(ret, ctx.src.get(kind).pos);
    let (vis, attr) = read_attrs(ctx, sum, it.vis, ctx.src.get_attached(id))?;

    let dummy_expr = hir::Expr {
      kind: hir::ExprKind::Const(hir::Const::Unit),
      category: hir::ExprCategory::RValue,
      pos: it.pos,
      ety: ctx.prims.ty_unit,
    }.push(ctx.cre);

    let method_item = hir::Item {
      name: Some(name),
      kind: hir::ItemKind::Function {
        expr: dummy_expr,
        kind: hir_kind,
      },
      path,
      vis, attr
    }.push(ctx.cre);

    Status::Success((method_item, hir_kind, loc, fctx))
  }

  pub(crate) fn finish_fun_field(ctx: &mut Ctx, sum: &mut Summary, it: &ast::Field, expr: ast::ExprId, method_item: hir::ItemId, hir_kind: hir::TypeId, mut loc: qwc_resolve::LocalScopeManager, fctx: &FunCtx) -> Status<()> {
    let expr_hir = ExprLow::low(ctx!(loc loc -> ctx), fctx, sum, expr)?;
    let expr_ty = ctx.cre.get(expr_hir).ety;

    let hir::TypeKind::Fun{ret, ..} = ctx.cre.get(hir_kind).kind else { unreachable!() };

    // Trait Patch
    let ret = if let hir::TypeKind::Trait{..} = ctx.get(ret).kind && let hir::TypeKind::TraitFrom{..} = ctx.get(expr_ty).kind {
      let fun_sign = ctx.cre.get_mut(hir_kind);

      if let hir::TypeKind::Fun { self_kind, args, .. } = fun_sign.kind {
        fun_sign.kind = hir::TypeKind::Fun { self_kind, args, ret: expr_ty };
      }

      expr_ty
    } else { ret };

    TypeMatch::matches(ctx, sum, ret, expr_ty, it.pos, None)?;

    if let hir::ItemKind::Function { expr: ref mut e, .. } = ctx.cre.get_mut(method_item).kind {
      *e = expr_hir;
    }

    Status::Success(())
  }

}


fn read_attrs(ctx: &Ctx, sum: &mut Summary, vis: ast::Visibility, attrs: Option<&Vec<Attribute>>) -> Status<(hir::ItemVis, ItemAttr)> {
  let mut ivis: Option<(ast::Ident, hir::SymVis)> = None;
  let mut attr = ItemAttr::empty();

  if let Some(attrs) = attrs {
    for key in attrs {
      let key = key.ident;
      
      match () {
        // C
        _ if key.sid() == ctx.sin.sid_c() => {
          if attr.contains(ItemAttr::C) {
            ret_err!(sum, Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //.add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            );
          };
          attr |= ItemAttr::C;
        }

        // Import
        _ if key.sid() == ctx.sin.sid_import() => {
          if let Some((pos, _)) = ivis {
            ret_err!(sum, Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            );
          };
          ivis = Some((key, hir::SymVis::Import))
        }

        // Export
        _ if key.sid() == ctx.sin.sid_export() => {
          if let Some((pos, _)) = ivis {
            ret_err!(sum, Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            );
          };
          ivis = Some((key, hir::SymVis::Export))
        }

        // Entry
        _ if key.sid() == ctx.sin.sid_entry() => {
          if let Some((pos, _)) = ivis {
            ret_err!(sum, Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            );
          };
          if attr.contains(ItemAttr::Entry) {
            ret_err!(sum, Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //.add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            );
          };
          if vis != ast::Visibility::Public {
            ret_err!(sum, Message::error(ENTRY_FUNCTION_MUST_BE_PUBLIC, Label::new_pos(key)));
          }
          ivis = Some((key, hir::SymVis::Export));
          attr |= ItemAttr::Entry;
        }
      
        _ => ret_err!(sum, Message::error(UNKNOWN_ATTRIBUTE.args(&[key.str(ctx.far)]), Label::new_pos(key)))
      }
    }
  }


  let vis = match vis {
    ast::Visibility::Inherited => hir::ItemVis::Private,
    ast::Visibility::Public  => hir::ItemVis::Public(ivis.map(|v| v.1).unwrap_or(hir::SymVis::Internal)),
    ast::Visibility::Private => hir::ItemVis::Private,
    _ => panic!()
  };

  Status::Success((vis, attr))
}
