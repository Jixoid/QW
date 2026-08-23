use crate::hir::{Rng, TypeId};


pub enum Thing {
  Name(u32),
  List(Rng),

  NamedType(u32, TypeId),
}
