use qwc_hir::{self as hir, Value};


pub trait Evaluator {
  fn evaluate(self, hir: &hir::Krate) -> Option<Value>;
}


impl Evaluator for hir::ExprId {
  fn evaluate(self, hir: &hir::Krate) -> Option<Value> {
    let it: &hir::Expr = hir.get(self);

    let val = match it.kind {
      hir::ExprKind::Lit(v) => v,

      _ => todo!("cannot evaluate yet")
    };

    Some(val)
  }
}
