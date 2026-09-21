use qwc_diagnostic::Message;
use qwc_hir as hir;
use qwc_mir::{self as mir, Value};

use crate::{BlockBuilder, Ctx, SymbLow, type_p::TypeLow};


pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, bbld: &mut BlockBuilder, id: hir::ExprId) -> Result<Option<Value>, Message> {
    let it: &hir::Expr = ctx.src.get(id);

    let it = match it.kind {
      // Value
      hir::ExprKind::Const(val) => Some(Self::low_const(ctx, val)?),

      hir::ExprKind::GlobalRef(item) => Some(Self::low_global_ref(ctx, item)?),
      
      
      // Non Return Expr
      hir::ExprKind::Assign{lhs, rhs} => {Self::low_assign(ctx, bbld, lhs, rhs)?; None},

      c @_ => todo!("{c:#?}")
    };

    Ok(it)
  }


  fn low_const(_ctx: &mut Ctx, val: hir::Const) -> Result<Value, Message> {
    let cons = match val {
      hir::Const::Unit => mir::Const::Unit,
      
      _ => todo!("{val:?}"),
    };

    
    // Post
    let this = Value::Const(
      cons
    );

    Ok(this)
  }

  fn low_global_ref(ctx: &mut Ctx, item: hir::ItemId) -> Result<Value, Message> {
    let item = SymbLow::low(ctx, item)?.unwrap();

    // Post
    let this = Value::GlobalRef(
      item
    );

    Ok(this)
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

}
