use crate::{Rng, SSA};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
  pub insts: Rng, /* InstId */
  pub stack: Rng, /* TypeId */
  pub ret: Option<SSA>,
}
