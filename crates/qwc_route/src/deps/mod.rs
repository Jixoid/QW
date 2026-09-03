use std::path::PathBuf;

use clap::Subcommand;
use owo_colors::OwoColorize;

pub mod sync;
pub mod add;
pub mod rm;


#[derive(Subcommand)]
pub enum DepsCommands {
  #[command(alias = "s")]
  /// Sync dependencies
  Sync {
    /// Project path
    #[arg(long, default_value = ".")]
    path: PathBuf,
  },

  #[command(alias = "a")]
  /// Add a new dependency 
  Add {
    /// Project path
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Name of the dependency
    name: String,

    /// Git URL of the dependency
    url: String,
  },

  #[command(alias = "r")]
  /// Remove a dependency
  Rm {
    /// Project path
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Name of the dependency to remove
    name: String,
  },
}

pub fn deps_cmd(cmd: DepsCommands) {
  match cmd {

    DepsCommands::Sync{path} => {
      let info = sync::SyncInfo {
        path: path.to_str().unwrap_or("").to_string(),
      };

      match sync::sync(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }

    DepsCommands::Add{path, name, url} => {
      let info = add::AddInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name,
        url,
      };

      match add::add(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }

    DepsCommands::Rm{path, name} => {
      let info = rm::RmInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name,
      };

      match rm::rm(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }
  
  }
}
