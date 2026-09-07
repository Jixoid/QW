use qwc_diagnostic::Message;
use qwc_ast as ast;
use qwc_hir::{self as hir};

use crate::{Ctx, ExprLow, TypeLow, ctx};


pub struct ItemLow;

impl ItemLow {

  pub fn low(ctx: &mut Ctx, id: ast::ItemId) -> Result<Option<hir::ItemId>, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let it: &ast::Item = src.get(id);
    
    let it = match it.kind {
      ast::ItemKind::Krate(rng) => Some(Self::low_krate(ctx, id, rng)?),

      ast::ItemKind::Module(rng) | ast::ItemKind::ModuleFile(rng, ..) => Some(Self::low_module(ctx, id, it, rng)?),
      
      ast::ItemKind::Generic{ctn: rng, ..} => Some(Self::low_generic(ctx, id, rng)?),


      // Symbols
      ast::ItemKind::Fun{kind, blok} => Some(Self::low_fun(ctx, it, kind, blok)?),

      ast::ItemKind::Let{kind, value, ism} => Some(Self::low_let(ctx, it, kind, value, ism)?),
      
      // Side Effect
      ast::ItemKind::Using(kind) | ast::ItemKind::ItemTy(kind) => {TypeLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), kind)?; None},
      
      // Unexpected
      ast::ItemKind::ModuleUnloaded | ast::ItemKind::ImplIn{..} => panic!("ast object that should not be present"),
      
      // Ignore
      ast::ItemKind::Impl{..} => None,
      ast::ItemKind::Import(..) => None,
      
      _ => todo!("{:#?}", it)
    };

    Ok(it)
  }


  fn low_krate(ctx: &mut Ctx, id: ast::ItemId, rng: ast::Rng) -> Result<hir::ItemId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let lscp = scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in src.extra_get(rng) {
        let id = ast::ItemId::new_from(id);
        
        Self::low(ctx!(cre,sum,src,sin,far,scp,lscp), id)?.map(|id| ctn.push(id));
      }

      cre.extra(&ctn)
    };


    // Post
    let this = hir::Item::RootNS {
      rng,
    };
    
    Ok(cre.push(this))
  }

  fn low_module(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::Rng) -> Result<hir::ItemId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let lscp = scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in src.extra_get(rng) {
        let id = ast::ItemId::new_from(id);
        
        Self::low(ctx!(cre,sum,src,sin,far,scp,lscp), id)?.map(|id| ctn.push(id));
      }

      cre.extra(&ctn)
    };


    // Post
    let this = hir::Item::NameSpace {
      name: it.name.unwrap().sid(),
      rng,
    };
    
    Ok(cre.push(this))
  }

  fn low_generic(ctx: &mut Ctx, id: ast::ItemId, rng: ast::Rng) -> Result<hir::ItemId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let lscp = scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in src.extra_get(rng) {
        let id = ast::ItemId::new_from(id);
        
        Self::low(ctx!(cre,sum,src,sin,far,scp,lscp), id)?.map(|id| ctn.push(id));
      }

      cre.extra(&ctn)
    };

    
    // Post
    let this = hir::Item::GenericNS {
      rng
    };

    Ok(cre.push(this))
  }

  fn low_fun(ctx: &mut Ctx, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let kind = TypeLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), kind)?;

    let expr = ExprLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), expr.unwrap())?;

    
    // Post
    let this = hir::Item::Function {
      name: it.name.unwrap().sid(),
      expr,
      kind,
    };

    Ok(cre.push(this))
  }

  fn low_let(ctx: &mut Ctx, it: &ast::Item, kind: Option<ast::TypeId>, expr: ast::ExprId, ism: bool) -> Result<hir::ItemId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let kind = kind.unwrap();
    let kind = TypeLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), kind)?;

    let expr = ExprLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), expr)?;


    // Post
    let this = hir::Item::Variable {
      name: it.name.unwrap().sid(),
      kind,
      expr,
      ism,
    };

    Ok(cre.push(this))
  }

}
