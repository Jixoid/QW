use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, id::PushOkApi};

use crate::{Ctx, Layouter};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: hir::TypeId) -> Result<mir::TypeId, Message> {
    if let Some(&id) = ctx.cmap.cache_type.get(&id) { return Ok(id) }

    let it = ctx.src.get(id);

    let ty = match it.kind {
      // Primitive
      hir::TypeKind::Unit => ctx.tin.ty_unit(),
      
      hir::TypeKind::Int(len, _) => Self::low_int(ctx, len)?,
      hir::TypeKind::Float(len) => Self::low_float(ctx, len)?,
      hir::TypeKind::Bool => ctx.tin.ty_bool(),

      hir::TypeKind::Ref(id, _) => Self::low_ref(ctx, id)?,

      // Combinated
      hir::TypeKind::Struct(rng) => Self::low_struct(ctx, rng)?,
      hir::TypeKind::Iface(rng) => Self::low_iface(ctx, rng)?,
      hir::TypeKind::TraitFrom{hidden, ..} => Self::low_trait_from(ctx, hidden)?,
      
      // Callable
      hir::TypeKind::Fun{self_kind, args, ret} => Self::low_fun(ctx, self_kind, args, ret)?,
      
      // Logic Error
      hir::TypeKind::Trait(..) => panic!("this type must not have infiltrated this layer!"),

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

  fn low_float(ctx: &mut Ctx, len: u16) -> Result<mir::TypeId, Message> {
    let it = match len {
      16  => ctx.tin.ty_f16(),
      32  => ctx.tin.ty_f32(),
      64  => ctx.tin.ty_f64(),
      128 => ctx.tin.ty_f128(),
      _ => panic!()
    };

    Ok(it)
  }


  fn low_ref(ctx: &mut Ctx, id: hir::TypeId) -> Result<mir::TypeId, Message> {
    let it = ctx.src.get(id);

    let it = match it.kind {
      hir::TypeKind::Iface(..) => {
        let ptr = ctx.tin.ty_ptr();
        
        let rng = ctx.cre.extra(&[ptr, ptr]);
        

        // Post
        let kind = mir::TypeKind::Struct(rng);

        mir::Type {
          kind,
          layout: Layouter::layout(&kind, ctx.tin.layinfo, ctx.cre, None),
        }.push(ctx.cre)
      }
      
      _ => ctx.tin.ty_ptr(),
    };

    Ok(it)
  }


  fn low_struct(ctx: &mut Ctx, rng: hir::ThingRng) -> Result<mir::TypeId, Message> {
    let rng = {
      let mut sub = vec![];

      for id in ctx.src.extra_get(rng) {
        let hir::Thing::NamedType(_, kind) = *ctx.src.get(id) else { panic!() };

        let kind = TypeLow::low(ctx, kind)?;

        sub.push(kind);
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

  fn low_iface(_ctx: &mut Ctx, _rng: hir::ThingRng) -> Result<mir::TypeId, Message> {
    todo!()
  }

  fn low_trait_from(ctx: &mut Ctx, hidden: hir::TypeId) -> Result<mir::TypeId, Message> {
    TypeLow::low(ctx, hidden)
  }

  fn low_fun(ctx: &mut Ctx, self_kind: Option<hir::TypeId>, args: hir::ThingRng, ret: hir::TypeId) -> Result<mir::TypeId, Message> {
    let args = {
      let mut ctn = vec![];

      if let Some(self_kind) = self_kind {
        ctn.push(Self::low(ctx, self_kind)?);
      }
      
      for id in ctx.src.extra_get(args) {
        let hir::Thing::NamedType(_, kind) = *ctx.src.get(id) else { panic!() };

        ctn.push(Self::low(ctx, kind)?);
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
