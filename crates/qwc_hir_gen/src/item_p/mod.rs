/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Label, Message, msg::*};
use qwc_ast::{self as ast, Attribute};
use qwc_hir::{self as hir, ItemAttr, PushOkApi};

use crate::{Ctx, ExprLow, TypeLow, ctx, TypeMatch};

mod impl_p;
mod ns_p;



pub struct ItemLow;

impl ItemLow {

  pub fn low(ctx: &mut Ctx, id: ast::ItemId) -> Result<Option<hir::ItemId>, Message> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);

    let it = match it.kind {
      // Scope
      ast::ItemKind::Krate(rng) => Some(ns_p::low_krate(ctx, id, it, rng)?),

      ast::ItemKind::Module(rng) | ast::ItemKind::ModuleFile(rng, ..) => Some(ns_p::low_module(ctx, id, it, rng)?),
      
      ast::ItemKind::Generic{ctn: rng, ..} => Some(ns_p::low_generic(ctx, id, it, rng)?),


      // Symbols
      ast::ItemKind::Fun{kind, blok} => Some(Self::low_fun(ctx, id, it, kind, blok)?),
      
      ast::ItemKind::Task{kind, blok} => Some(Self::low_task(ctx, id, it, kind, blok)?),
      
      ast::ItemKind::Let{kind, value, ism} => Some(Self::low_let(ctx, id, it, kind, value, ism)?),
      
      ast::ItemKind::Impl{type_ty, trait_ty, ctn} => Some(impl_p::low_impl(ctx, id, it, type_ty, trait_ty, ctn)?),


      // Using
      ast::ItemKind::Using(kind) | ast::ItemKind::ItemTy(kind) => Some(Self::low_using(ctx, id, it, kind)?),


      // Unexpected
      ast::ItemKind::ModuleUnloaded => panic!("ast object that should not be present"),
      
      // Impl
      ast::ItemKind::Import(..) => None,
    };

    ctx.cmap.cache_item.insert(id, it);

    Ok(it)
  }


  fn low_using(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Result<hir::ItemId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Using {
        name: it.name.unwrap().sid(),
        kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }


  fn low_let(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: Option<ast::TypeId>, expr: ast::ExprId, ism: bool) -> Result<hir::ItemId, Message> {
    let kind = kind.unwrap();
    let kind = TypeLow::low(ctx, kind)?;

    let expr = ExprLow::low(ctx, expr)?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;
    

    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Variable {
        name: it.name.unwrap().sid(),
        kind,
        expr,
        ism,
      },
      vis, attr
    };

    Ok(ctx.cre.push(this))
  }


  fn low_fun(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ret} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      let ast::Thing::NamedType(name_pos, ..) = *ctx.src.get(id_pos) else { panic!() };

      loc.insert(name, kind, false, name_pos);
    }

    let expr_hir = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;
    
    let expr_pos = ctx.src.get(expr.unwrap()).pos;
    let expr_ty = ctx.cre.get(expr_hir).ety;

    let ret = if let hir::TypeKind::Trait(..) = ctx.get_type(ret).kind && let hir::TypeKind::TraitFrom{..} = ctx.get_type(expr_ty).kind {
      let fun_sign = ctx.cre.get_mut(hir_kind);

      let hir::TypeKind::Fun {ret: sign_ret, ..} = &mut fun_sign.kind else { panic!() };

      *sign_ret = expr_ty;
      expr_ty
    } else { ret };

    TypeMatch::matches_pos(ctx, ret, expr_ty, expr_pos)?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Function {
        name: it.name.unwrap().sid(),
        expr: expr_hir,
        kind: hir_kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }

  fn low_task(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ..} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      let ast::Thing::NamedType(name_pos, ..) = *ctx.src.get(id_pos) else { panic!() };

      loc.insert(name, kind, false, name_pos);
    }

    let expr = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Task {
        name: it.name.unwrap().sid(),
        expr,
        kind: hir_kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }

  fn low_fun_field(ctx: &mut Ctx, id: ast::FieldId, it: &ast::Field, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let hir_kind = TypeLow::low(ctx, kind)?;
    
    let mut loc = qwc_resolve::LocalScopeManager::new();

    // Args
    let ast::TypeKind::Fun{self_kind: self_ast, args: args_ast, ..} = ctx.src.get(kind).kind else { unreachable!() };
    let hir::TypeKind::Fun{self_kind, args, ..} = ctx.cre.get(hir_kind).kind else { unreachable!() };
    
    if let Some(ty) = self_kind {
      let self_pos = ctx.src.get(self_ast.unwrap()).pos;
      
      loc.insert(ctx.sin.sid_self(), ty, false, self_pos);
    }

    for (id, id_pos) in ctx.cre.extra_get(args).zip(ctx.src.extra_get(args_ast)) {
      let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) else { panic!() };
      let ast::Thing::NamedType(name_pos, ..) = *ctx.src.get(id_pos) else { panic!() };

      loc.insert(name, kind, false, name_pos);
    }

    let expr = ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?;

    let (vis, attr) = read_attrs(ctx, it.vis, ctx.src.get_attached(id))?;


    // Post
    hir::Item {
      kind: hir::ItemKind::Function {
        name: it.name.unwrap().sid(),
        expr,
        kind: hir_kind,
      },
      vis, attr
    }.push_ok(ctx.cre)
  }

}


fn read_attrs(ctx: &Ctx, vis: ast::Visibility, attrs: Option<&Vec<Attribute>>) -> Result<(hir::ItemVis, ItemAttr), Message> {
  let mut ivis: Option<(ast::Ident, hir::SymVis)> = None;
  let mut attr = ItemAttr::empty();


  if let Some(attrs) = attrs {
    for key in attrs {
      let key = key.ident;
      
      match () {
        // C
        _ if key.sid() == ctx.sin.sid_c() => {
          if attr.contains(ItemAttr::C) {
            return Err(Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //.add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          attr |= ItemAttr::C;
        }

        // Import
        _ if key.sid() == ctx.sin.sid_import() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Import))
        }

        // Export
        _ if key.sid() == ctx.sin.sid_export() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Export))
        }

        // Entry
        _ if key.sid() == ctx.sin.sid_entry() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          if attr.contains(ItemAttr::Entry) {
            return Err(Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //.add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          if vis != ast::Visibility::Public {
            return Err(Message::error(ENTRY_FUNCTION_MUST_BE_PUBLIC, Label::new_pos(key)))
          }
          ivis = Some((key, hir::SymVis::Export));
          attr |= ItemAttr::Entry;
        }
      
        _ => return Err(Message::error(UNKNOWN_ATTRIBUTE.args(&[key.str(ctx.far)]), Label::new_pos(key)))
      }
    }
  }



  let vis = match vis {
    ast::Visibility::Inherited => hir::ItemVis::Private,
    ast::Visibility::Public  => hir::ItemVis::Public(ivis.unwrap() /* TODO! */ .1),
    ast::Visibility::Private => hir::ItemVis::Private,
    _ => panic!()
  };

  Ok((vis, attr))
}
