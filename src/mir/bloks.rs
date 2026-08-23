use crate::mir::BlokId;


pub struct Blok {
  pub name: String,
  pub instrs: Vec<BlokId>,
}
