use crate::Rng;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
  pub stack: Rng, /* TypeId */
}
