use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Layouter};

use crate::{Ctx, ctx, layout::LayouterQW};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: hir::TypeId) -> Result<mir::TypeId, Message> { ctx!(ctx => cre, tin, sum, src, sin, mgr);
    let it: &hir::Type = src.get(id);

    let ty = match *it {
      hir::Type::Unit => tin.ty_unit(),

      hir::Type::Ref(id) => {let id = Self::low(ctx!(cre,tin,sum,src,sin,mgr), id)?; tin.ty_ptr(cre, id)},

      hir::Type::Fun{args, ret} => Self::low_fun(ctx, args, ret)?,

      kind @_ => todo!("{kind:#?}")
    };

    Ok(ty)
  }


  fn low_fun(ctx: &mut Ctx, rng: hir::Rng, ret: hir::TypeId) -> Result<mir::TypeId, Message> { ctx!(ctx => cre, tin, sum, src, sin, mgr);
    let args = {
      let mut ctn = vec![];
      
      for id in src.extra_get(rng) {
        let id = hir::TypeId::new_from(id);
        
        ctn.push(Self::low(ctx!(cre,tin,sum,src,sin,mgr), id)?);
      }

      cre.extra(&ctn)
    };

    let ret = Self::low(ctx!(cre,tin,sum,src,sin,mgr), ret)?;


    // Post
    let kind = mir::TypeKind::Fun{
      args, ret,
    };

    let this = mir::Type{
      kind,
      layout: LayouterQW::layout(&kind, tin.layinfo, cre),
    };

    
    Ok(cre.push(this))
  }

}
