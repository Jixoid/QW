use qwc_diagnostic::{Label, Message, msg::*};
use qwc_ast::{self as ast, Attribute};
use qwc_hir::{self as hir};
use qwc_string_interner::StrInterner;

use crate::{Ctx, ExprLow, TypeLow, ctx};


pub struct ItemLow;

impl ItemLow {

  pub fn low(ctx: &mut Ctx, id: ast::ItemId) -> Result<Option<hir::ItemId>, Message> {
    if let Some(&id) = ctx.cmap.cache_item.get(&id) { return Ok(id) }

    let it: &ast::Item = ctx.src.get(id);

    let it = match it.kind {
      ast::ItemKind::Krate(rng) => Some(Self::low_krate(ctx, id, it, rng)?),

      ast::ItemKind::Module(rng) | ast::ItemKind::ModuleFile(rng, ..) => Some(Self::low_module(ctx, id, it, rng)?),
      
      ast::ItemKind::Generic{ctn: rng, ..} => Some(Self::low_generic(ctx, id, it, rng)?),

      
      ast::ItemKind::Using(kind) | ast::ItemKind::ItemTy(kind) => Some(Self::low_using(ctx, id, it, kind)?),
      


      // Symbols
      ast::ItemKind::Fun{kind, blok} => Some(Self::low_fun(ctx, id, it, kind, blok)?),

      ast::ItemKind::Let{kind, value, ism} => Some(Self::low_let(ctx, id, it, kind, value, ism)?),
      
      // Unexpected
      ast::ItemKind::ModuleUnloaded => panic!("ast object that should not be present"),
      
      // Ignore
      ast::ItemKind::Impl{..} => None,
      ast::ItemKind::Import(..) => None,
    };

    ctx.cmap.cache_item.insert(id, it);

    Ok(it)
  }


  fn low_krate(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        Self::low(ctx!(lscp -> ctx), id)?.map(|id| ctn.push(id));
      }

      ctx.cre.extra(&ctn)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::RootNS {
        rng,
      },
      vis: convert_vis(it.vis),
      svis
    };
    
    Ok(ctx.cre.push(this))
  }

  fn low_module(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        Self::low(ctx!(lscp -> ctx), id)?.map(|id| ctn.push(id));
      }

      ctx.cre.extra(&ctn)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::NameSpace {
        name: it.name.unwrap().sid(),
        rng,
      },
      vis: convert_vis(it.vis),
      svis
    };
    
    Ok(ctx.cre.push(this))
  }

  fn low_generic(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, rng: ast::ItemRng) -> Result<hir::ItemId, Message> {
    let lscp = ctx.scp.get(&id.to_any()).unwrap();

    let rng = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(rng) {
        Self::low(ctx!(lscp -> ctx), id)?.map(|id| ctn.push(id));
      }

      ctx.cre.extra(&ctn)
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;

    
    // Post
    let this = hir::Item {
      kind: hir::ItemKind::GenericNS {
        rng
      },
      vis: convert_vis(it.vis),
      svis
    };

    Ok(ctx.cre.push(this))
  }


  fn low_using(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId) -> Result<hir::ItemId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;


    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Using {
        name: it.name.unwrap().sid(),
        kind,
      },
      vis: convert_vis(it.vis),
      svis,
    };

    Ok(ctx.cre.push(this))
  }


  fn low_fun(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: ast::TypeId, expr: Option<ast::ExprId>) -> Result<hir::ItemId, Message> {
    let kind = TypeLow::low(ctx, kind)?;

    let mut loc = qwc_resolve::LocalScopeManager::new();
    let expr = {
      ExprLow::low(ctx!(loc loc -> ctx), expr.unwrap())?
    };

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;

    
    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Function {
        name: it.name.unwrap().sid(),
        expr,
        kind,
      },
      vis: convert_vis(it.vis),
      svis
    };

    Ok(ctx.cre.push(this))
  }

  fn low_let(ctx: &mut Ctx, id: ast::ItemId, it: &ast::Item, kind: Option<ast::TypeId>, expr: ast::ExprId, ism: bool) -> Result<hir::ItemId, Message> {
    let kind = kind.unwrap();
    let kind = TypeLow::low(ctx, kind)?;

    let expr = ExprLow::low(ctx, expr)?;

    let svis = read_attrs(ctx.sin, ctx.src.get_attached(id))?;
    

    // Post
    let this = hir::Item {
      kind: hir::ItemKind::Variable {
        name: it.name.unwrap().sid(),
        kind,
        expr,
        ism,
      },
      vis: convert_vis(it.vis),
      svis,
    };

    Ok(ctx.cre.push(this))
  }

}


fn read_attrs(sin: &StrInterner, attrs: Option<&Vec<Attribute>>) -> Result<Option<hir::SymVis>, Message> {
  let mut ivis: Option<(ast::Ident, hir::SymVis)> = None;

  if let Some(attrs) = attrs {
    for attr in attrs {
      let key = attr.ident;
      
      match () {
        _ if key.sid() == sin.sid_import() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Import))
        },

        _ if key.sid() == sin.sid_export() => {
          if let Some((pos, _)) = ivis {
            return Err(Message::error(MUTUALLY_CONTRADICTORY_DEFINITIONS, Label::new(key, CONFLICTING_DEFINITION))
              .add(Label::new(pos, FIRST_DEFINITION_HERE))
              .add(ONLY_ONE_DEFINITION_REMAIN)
            )
          };
          ivis = Some((key, hir::SymVis::Export))
        },
      
        _ => return Err(Message::error(UNKNOWN_ATTRIBUTE, Label::new_pos(key)))
      }
    }
  }

  Ok(ivis.map(|val| val.1))
}


fn convert_vis(vis: ast::Visibility) -> hir::ItemVis {
  match vis {
    ast::Visibility::Inherited => hir::ItemVis::Private,
    ast::Visibility::Public  => hir::ItemVis::Public,
    ast::Visibility::Private => hir::ItemVis::Private,
    _ => panic!()
  }
}
