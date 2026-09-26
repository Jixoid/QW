use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{Ctx, Layouter};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: hir::TypeId) -> Result<mir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }

    let it: &hir::Type = ctx.src.get(id);

    let ty = match it.kind {
      // ZST
      hir::TypeKind::Unit => ctx.tin.ty_unit(),

      // Primitive
      hir::TypeKind::Ref(id, _) => {Self::low(ctx, id)?; ctx.tin.ty_ptr()},

      hir::TypeKind::Int(len, _) => Self::low_int(ctx, len)?,
      hir::TypeKind::Bool => ctx.tin.ty_bool(),

      // Combinated
      hir::TypeKind::Struct(rng) => Self::low_struct(ctx, rng)?,

      // Callable
      hir::TypeKind::Fun{args, ret} => Self::low_fun(ctx, args, ret)?,

      kind @_ => todo!("{kind:#?}")
    };

    ctx.cmap.cache_type.insert(id, ty);

    Ok(ty)
  }


  fn low_int(ctx: &mut Ctx, len: u16) -> Result<mir::TypeId, Message> {
    let it = match len {
      8   => ctx.tin.ty_i8(),
      16  => ctx.tin.ty_i16(),
      32  => ctx.tin.ty_i32(),
      64  => ctx.tin.ty_i64(),
      128 => ctx.tin.ty_i128(),
      _ => panic!()
    };

    Ok(it)
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
      layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre, None),
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
      layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre, None),
    };

    
    Ok(ctx.cre.push(this))
  }

}
