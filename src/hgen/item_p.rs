use std::assert_matches;

use crate::{ast::{self, AstKind, Rng}, diagnostic::Message, hgen::{DeclGen, GenContext}, hir};


pub struct ItemGen;

impl ItemGen {

  pub fn low(ctx: &mut GenContext, id: ast::ItemId) -> Result<hir::TypeId, Message> {
    let it = ctx.ast.get_item(id);

    match &it.vari {
      ast::ItemVari::Module{name, ctn} => Self::low_module(ctx, name, ctn),
      ast::ItemVari::Generic{params, reqs, ctn} => Self::low_generic(ctx, params, reqs, ctn),
      ast::ItemVari::Impl{type_ty, trait_ty, ctn} => Self::low_impl(ctx, type_ty, trait_ty, ctn),

      _ => todo!("unknown item: {:?}", it.vari),
    }
  }


  fn low_module(ctx: &mut GenContext, name: &String, ctn: &Rng) -> Result<hir::TypeId, Message> {
    println!("ITEM.MODULE {:?} [{:?}]", name, ctn);

    let exda = ctx.ast.get_extra(ctn.clone());
    
    for x in exda {
      assert_matches!(x.kind, AstKind::Decl | AstKind::Item);

      match x.kind {
        AstKind::Decl => DeclGen::low(ctx, ast::DeclId::new_from(*x))?,
        AstKind::Item => ItemGen::low(ctx, ast::ItemId::new_from(*x))?,
        _ => panic!(),
      };
    }

    panic!()
  }

  fn low_generic(ctx: &mut GenContext, _params: &Rng, reqs: &Rng, ctn: &Rng) -> Result<hir::TypeId, Message> {
    println!("ITEM.GENERIC reqs[{:?}] [{:?}]", reqs, ctn);

    let exda = ctx.ast.get_extra(ctn.clone());

    for x in exda {
      assert_matches!(x.kind, AstKind::Decl | AstKind::Item);

      match x.kind {
        AstKind::Decl => DeclGen::low(ctx, ast::DeclId::new_from(*x))?,
        AstKind::Item => ItemGen::low(ctx, ast::ItemId::new_from(*x))?,
        _ => panic!(),
      };
    }

    panic!()
  }

  fn low_impl(_ctx: &mut GenContext, type_ty: &ast::TypeId, trait_ty: &Option<ast::TypeId>, ctn: &Rng) -> Result<hir::TypeId, Message> {
    println!("ITEM.IMPL {}: {:?} [{:?}]", type_ty, trait_ty, ctn);

    panic!()
  }

}
