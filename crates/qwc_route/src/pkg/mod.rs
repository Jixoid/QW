/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::PathBuf;

use owo_colors::OwoColorize;
use clap::Subcommand;

pub mod install;


#[derive(Subcommand)]
pub enum PkgCommands {
  #[command(alias = "i")]
  /// Install to system
  Install {
    /// Project path
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Install to user local
    #[arg(short, long)]
    force: bool
  }
}


pub fn pkg_cmd(cmd: PkgCommands) {
  match cmd {

    PkgCommands::Install{path, force} => {
      let info = install::InstallInfo {
        path: path.to_str().unwrap_or("").to_string(),
        force,
      };

      match install::install(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }

  }
}
