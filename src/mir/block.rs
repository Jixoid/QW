use super::identy::HirId;
use core::fmt;
use owo_colors::OwoColorize;


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

impl fmt::Display for HirBlock {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}{} ", "block".blue().bold(), self.name.white().bold(), ":".bright_black())?;
    write!(f, "{}", "[".bright_black())?;
    for (i, instr) in self.instrs.iter().enumerate() {
      write!(f, "{}", instr)?;
      if i + 1 < self.instrs.len() {
        write!(f, "{} ", ",".bright_black())?;
      }
    }
    write!(f, "{}", "]".bright_black())
  }
}
