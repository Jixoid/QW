use super::identy::HirId;
use core::fmt;
use owo_colors::OwoColorize;

#[derive(Debug, Clone, PartialEq)]
pub enum HirValue {
  ConstInt(i64),
  ConstFloat(f64),
  ConstBool(bool),
  Reg(HirId),
  Global(HirId),
  Null,
}

impl fmt::Display for HirValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      HirValue::ConstInt(i) => write!(f, "{}", i.yellow()),
      HirValue::ConstFloat(fl) => write!(f, "{}", fl.yellow()),
      HirValue::ConstBool(b) => write!(f, "{}", b.yellow()),
      HirValue::Reg(id) => write!(f, "{}{}", "%".bright_black(), id.index().cyan()),
      HirValue::Global(id) => write!(f, "{}{}", "@".bright_black(), id.index().cyan()),
      HirValue::Null => write!(f, "{}", "null".bright_black()),
    }
  }
}
