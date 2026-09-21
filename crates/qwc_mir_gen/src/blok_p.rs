use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{BlockBuilder, Ctx, ExprLow};


pub struct BlokLow;

impl BlokLow {

  pub fn low(ctx: &mut Ctx, id: hir::ExprId) -> Result<mir::BlokId, Message> {
    if let Some(&id) = ctx.cmap.cache_blok.get(&id) { return Ok(id) }

    let it: &hir::Expr = ctx.src.get(id);

    let ex = match it.kind {
      hir::ExprKind::Block{stmt, expr} => Self::low_block(ctx, stmt, expr)?,

      kind @_ => todo!("{kind:#?}")
    };

    ctx.cmap.cache_blok.insert(id, ex);

    Ok(ex)
  }


  fn low_block(ctx: &mut Ctx, stmt: hir::Rng, expr: Option<hir::ExprId>) -> Result<mir::BlokId, Message> {
    let mut bbld = BlockBuilder::new();

    for id in ctx.src.extra_get(stmt) {
      let id = hir::ExprId::new_from(id);
      ExprLow::low(ctx, &mut bbld, id)?;
    }


    if let Some(expr) = expr {
      let ret = ExprLow::low(ctx, &mut bbld, expr)?;

      ret.map(|ret| {
        // Post
        let this = mir::Expr::Return(ret);

        bbld.emit(this)
      });
    };

    let insts = {
      let mut insts = vec![];
      
      for &inst in &bbld.insts {
        let id = ctx.cre.push(inst);
        
        insts.push(id);
      }

      ctx.cre.extra(&insts)
    };
    
    let stack = ctx.cre.extra(&bbld.stack);


    // Post
    let this = mir::Block {
      insts,
      stack,
    };

    Ok(ctx.cre.push(this))
  }

}
