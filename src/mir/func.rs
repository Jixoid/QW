use super::identy::HirId;

#[derive(Debug, Clone, PartialEq)]
pub struct HirFunc {
  pub name: String,
  pub ret_ty: HirId,
  pub arg_tys: Vec<HirId>,
  pub blocks: Vec<HirId>,
  pub is_weak: bool,
}

impl HirFunc {
  pub fn new(name: String, ret_ty: HirId, arg_tys: Vec<HirId>, is_weak: bool) -> Self {
    Self {
      name,
      ret_ty,
      arg_tys,
      blocks: Vec::new(),
      is_weak,
    }
  }

  pub fn push_block(&mut self, block: HirId) {
    self.blocks.push(block);
  }
}

use core::fmt;
use owo_colors::OwoColorize;

impl fmt::Display for HirFunc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let weak_str = if self.is_weak { " weak".green().bold().to_string() } else { "".to_string() };
    write!(f, "{}{} ", "fun".blue().bold(), weak_str)?;
    write!(f, "{}{}", self.name, "(".bright_black())?;
    for (i, arg) in self.arg_tys.iter().enumerate() {
      write!(f, "{}", arg)?;
      if i + 1 < self.arg_tys.len() { write!(f, "{} ", ",".bright_black())?; }
    }
    write!(f, "{} {}", ") ->".bright_black(), self.ret_ty)
  }
}
