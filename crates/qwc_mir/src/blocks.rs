use crate::{BlokId, InstRng, Value};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Block {
  pub insts: InstRng,
  pub term: Terminator,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Terminator {
  Jump(BlokId),
  Branch{cond: Value, then_bb: BlokId, else_bb: BlokId},
  Return(Option<Value>),
  Unreachable,
}
