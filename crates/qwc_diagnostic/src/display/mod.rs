use std::fmt;
use owo_colors::OwoColorize;
use qwc_arena::Files;
use crate::{Level, Message};

mod human;

#[cfg(feature = "lsp")]
mod lsp;


impl Message {
  pub fn display_human<'a>(&'a self, far: &'a Files) -> human::MessageHumanDisplay<'a> {
    human::MessageHumanDisplay(self, far)
  }
}


impl fmt::Display for Level {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Level::Fatal => write!(f, "{}", "fatal".red().bold()),
      Level::Error => write!(f, "{}", "error".red().bold()),
      Level::Warn  => write!(f, "{}", "warn".yellow().bold()),
      Level::Hint  => write!(f, "{}", "hint".yellow().bold()),
    }
  }
}
