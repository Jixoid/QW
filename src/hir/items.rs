use crate::hir::{ExprId, Rng, TypeId};


pub enum Item {
  Module(Rng /* ItemId */),

  Fun { kind: TypeId, blok: Option<ExprId> },

  Var { kind: TypeId, init: Option<ExprId> },
  Let { kind: TypeId, init: Option<ExprId> },
}
