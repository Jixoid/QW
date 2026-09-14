use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir};

use crate::{Ctx, ctx, TypeLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low(ctx: &mut Ctx, id: hir::ExprId) -> Result<mir::BlokId, Message> { ctx!(ctx => cre, tin, sum, src, sin, mgr);
    let it: &hir::Expr = src.get(id);

    let ex = match it.kind {
      hir::ExprKind::Block{stack, expr} => Self::low_block(ctx, stack, expr)?,

      kind @_ => todo!("{kind:#?}")
    };

    Ok(ex)
  }


  fn low_block(ctx: &mut Ctx, stack: hir::Rng, _expr: Option<hir::ExprId>) -> Result<mir::BlokId, Message> { ctx!(ctx => cre, tin, sum, src, sin, mgr);
    let stack = {
      let mut vec = vec![];

      for id in src.extra_get(stack) {
        let id = hir::TypeId::new_from(id);
        
        let id = TypeLow::low(ctx!(cre,tin,sum,src,sin,mgr), id)?;

        vec.push(id);
      }

      cre.extra(&vec)
    };
      

    // Post
    let this = mir::Block{
      stack,
    };
    
    Ok(cre.push(this))
  }

}
