use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir as mir;

use crate::{BlockBuilder, Ctx, SymbLow, type_p::TypeLow};


pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, bbld: &mut BlockBuilder, id: hir::ExprId) -> Result<Option<mir::SSA>, Message> {
    let it: &hir::Expr = ctx.src.get(id);

    let it = match it.kind {
      hir::ExprKind::Const(val) => Self::low_const(ctx, bbld, val)?,

      hir::ExprKind::Assign{lhs, rhs} => {Self::low_assign(ctx, bbld, lhs, rhs)?; None},

      hir::ExprKind::GlobalRef(item) => Self::low_global_ref(ctx, bbld, item)?,

      c @_ => todo!("{c:#?}")
    };

    Ok(it)
  }


  fn low_const(_ctx: &mut Ctx, bbld: &mut BlockBuilder, val: hir::Const) -> Result<Option<mir::SSA>, Message> {
    let cons = match val {
      hir::Const::Unit => mir::Const::Unit,
      
      _ => todo!("{val:?}"),
    };

    
    // Post
    let this = mir::Expr::Const(
      cons
    );

    Ok(bbld.emit(this))
  }


  fn low_assign(ctx: &mut Ctx, bbld: &mut BlockBuilder, lhs: hir::ExprId, rhs: hir::ExprId) -> Result<(), Message> {
    let target = ExprLow::low(ctx, bbld, lhs)?.unwrap();
    
    let kind = TypeLow::low(ctx, (ctx.src.get(lhs) as &hir::Expr).ety)?;

    let value = ExprLow::low(ctx, bbld, rhs)?.unwrap();

    
    // Post
    let this = mir::Expr::Store{
      target,
      kind,
      value,
    };
    
    bbld.emit(this);
    
    Ok(())
  }


  fn low_global_ref(ctx: &mut Ctx, bbld: &mut BlockBuilder, item: hir::ItemId) -> Result<Option<mir::SSA>, Message> {
    let item = SymbLow::low(ctx, item)?.unwrap();

    // Post
    let this = mir::Expr::GlobalRef(
      item
    );

    Ok(bbld.emit(this))
  }

}
