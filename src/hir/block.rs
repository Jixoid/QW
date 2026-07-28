use super::identy::HirId;

#[derive(Debug, Clone, PartialEq)]
pub struct HirBlock {
  pub instrs: Vec<HirId>,
  pub name: String,
}

impl HirBlock {
  pub fn new(name: String) -> Self {
    Self {
      instrs: Vec::new(),
      name,
    }
  }

  pub fn push_instr(&mut self, id: HirId) {
    self.instrs.push(id);
  }
}
