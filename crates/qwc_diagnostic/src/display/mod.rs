/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


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
