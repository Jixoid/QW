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

    let ret = match expr {
      Some(eid) => ExprLow::low(ctx, &mut bbld, eid)?,
      None => None,
    };

    let mut inst_ids = Vec::with_capacity(bbld.insts.len());
    for inst in bbld.insts {
      inst_ids.push(ctx.cre.push(inst));
    }
    let insts_rng = ctx.cre.extra(&inst_ids);
    let stack_rng = ctx.cre.extra(&bbld.stack);

    let this = mir::Block {
      insts: insts_rng,
      stack: stack_rng,
      ret,
    };

    Ok(ctx.cre.push(this))
  }

}
