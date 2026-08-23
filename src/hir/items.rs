use crate::hir::{Rng, TypeId};


pub enum Item {
  Module(Rng /* ItemId */),

  Fun(TypeId),

  Var(TypeId),
  Let(TypeId),
}
