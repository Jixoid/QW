use qwc_hir::{self as hir, Const};


pub trait Evaluator {
  fn evaluate(self, hir: &hir::Krate) -> Option<Const>;
}


impl Evaluator for hir::ExprId {
  fn evaluate(self, hir: &hir::Krate) -> Option<Const> {
    let it: &hir::Expr = hir.get(self);

    let val = match it.kind {
      hir::ExprKind::Const(v) => v,

      _ => todo!("cannot evaluate yet")
    };

    Some(val)
  }
}
