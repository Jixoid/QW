use super::identy::HirId;
use super::value::HirValue;

use owo_colors::OwoColorize;


#[derive(Debug, Clone, PartialEq)]
pub struct HirGlobalVar {
  pub name: String,
  pub ty: HirId,
  pub init: Option<HirValue>,
  pub is_const: bool,
  pub is_weak: bool,
}

impl HirGlobalVar {
  
  pub fn new(name: String, ty: HirId, init: Option<HirValue>, is_const: bool, is_weak: bool) -> Self {
    Self {name, ty, init, is_const, is_weak}
  }

}

use core::fmt;

impl fmt::Display for HirGlobalVar {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut modifiers = String::new();
    if self.is_const { modifiers.push_str(" imm"); } else { modifiers.push_str(" mut"); }
    
    if self.is_weak { modifiers.push_str(" weak"); }
    
    write!(f, "{}{} {}{} {} {} {:?}",
      "let".blue().bold(),
      format!("{}", modifiers).green().bold(),
      self.name,
      ":".bright_black(),
      self.ty,
      "=".bright_black(),
      self.init,
    )
  }
}
