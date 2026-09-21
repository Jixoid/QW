use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Layouter};

use crate::{Ctx, layout::LayouterQW};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: hir::TypeId) -> Result<mir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }

    let it: &hir::Type = ctx.src.get(id);

    let ty = match *it {
      hir::Type::Unit => ctx.tin.ty_unit(),

      hir::Type::Ref(id) => {Self::low(ctx, id)?; ctx.tin.ty_ptr()},

      hir::Type::Struct(rng) => Self::low_struct(ctx, rng)?,

      hir::Type::Fun{args, ret} => Self::low_fun(ctx, args, ret)?,

      kind @_ => todo!("{kind:#?}")
    };

    ctx.cmap.cache_type.insert(id, ty);

    Ok(ty)
  }


  fn low_struct(ctx: &mut Ctx, rng: hir::Rng) -> Result<mir::TypeId, Message> {
    let rng = {
      let mut sub = vec![];

      for id in ctx.src.extra_get(rng) {
        let id = hir::TypeId::new_from(id);

        let id = TypeLow::low(ctx, id)?;

        sub.push(id);
      }

      ctx.cre.extra(&sub)
    };
    

    // Post
    let kind = mir::TypeKind::Struct(rng);

    let this = mir::Type {
      kind,
      layout: LayouterQW::layout(&kind, ctx.tin.layinfo, ctx.cre),
    };

    Ok(ctx.cre.push(this))
  }


  fn low_fun(ctx: &mut Ctx, args: hir::Rng, ret: hir::TypeId) -> Result<mir::TypeId, Message> {
    let args = {
      let mut ctn = vec![];
      
      for id in ctx.src.extra_get(args) {
        let id = hir::TypeId::new_from(id);
        
        ctn.push(Self::low(ctx, id)?);
      }

      ctx.cre.extra(&ctn)
    };

    let ret = Self::low(ctx, ret)?;


    // Post
    let kind = mir::TypeKind::Fun{
      args, ret,
    };

    let this = mir::Type{
      kind,
      layout: LayouterQW::layout(&kind, ctx.tin.layinfo, ctx.cre),
    };

    
    Ok(ctx.cre.push(this))
  }

}
