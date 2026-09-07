use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir};

use crate::{Ctx, ctx};


pub struct TypeLow;

impl TypeLow {

  pub fn low(ctx: &mut Ctx, id: hir::TypeId) -> Result<mir::TypeId, Message> { ctx!(ctx => cre, sum, src, sin, mgr);
    let it: &hir::Type = src.get(id);

    let ty = match *it {
      hir::Type::Unit => cre.ty_unit(),

      hir::Type::Ref(id) => {let id = Self::low(ctx!(cre,sum,src,sin,mgr), id)?; cre.ty_ptr(id)},

      hir::Type::Fun{args, ret} => Self::low_fun(ctx, args, ret)?,

      kind @_ => todo!("{kind:#?}")
    };

    Ok(ty)
  }


  fn low_fun(ctx: &mut Ctx, rng: hir::Rng, ret: hir::TypeId) -> Result<mir::TypeId, Message> { ctx!(ctx => cre, sum, src, sin, mgr);
    let args = {
      let mut ctn = vec![];
      
      for id in src.extra_get(rng) {
        let id = hir::TypeId::new_from(id);
        
        ctn.push(Self::low(ctx!(cre,sum,src,sin,mgr), id)?);
      }

      cre.extra(&ctn)
    };

    let ret = Self::low(ctx!(cre,sum,src,sin,mgr), ret)?;


    // Post
    let this = mir::Type::Fun{
      args, ret,
    };
    
    Ok(cre.push(this))
  }

}
