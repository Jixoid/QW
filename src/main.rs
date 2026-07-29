use clap::{Parser, Subcommand};
use std::path::PathBuf;
use owo_colors::OwoColorize;

use crate::route::init;
use crate::route::build;
mod route;
mod control;
pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod front;
pub mod sema;
pub mod layout;
pub mod cgen;
pub mod basic_cgen;
pub mod sys;
pub mod ds;
pub mod hir;
pub mod hgen;


#[derive(Parser)]
#[command(name = "qw", version, about = "qw compiler")]
pub struct Cli {
  #[command(subcommand)]
  pub command: MainCommands,
}


#[derive(Subcommand)]
pub enum MainCommands {
  /// Create a new qw package
  Init {
    /// Directory to initialize
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Set the package name
    #[arg(short, long)]
    name: Option<String>,

    /// Set the package description
    #[arg(short, long)]
    desc: Option<String>,

    /// Do not initialize a new git repository
    #[arg(long)]
    no_git: bool,
  },

  /// Compile the package
  Build {
    /// Directory to build
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Display execution times for each compiler phase
    #[arg(long)]
    timings: bool,

    /// Use verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dump the parsed Abstract Syntax Tree to stdout
    #[arg(long)]
    ast_dump: bool,
    
    /// Dump the High-level Intermediate Representation to stdout
    #[arg(long)]
    hir_dump: bool,
  },

  /// Check the package for errors
  Check {
    /// Directory to check
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Display execution times for each compiler phase
    #[arg(long)]
    timings: bool,

    /// Use verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dump the parsed Abstract Syntax Tree to stdout
    #[arg(long)]
    ast_dump: bool,
  },

  /// Manage project dependencies
  Deps {
    #[command(subcommand)]
    command: DepsCommands,
  },
}


#[derive(Subcommand)]
pub enum DepsCommands {
  /// Sync dependencies
  Sync {
    /// Project path
    #[arg(long, default_value = ".")]
    path: PathBuf,
  },

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

  /// Remove a dependency
  Rm {
    /// Project path
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Name of the dependency to remove
    name: String,
  },
}



fn main() {
  let cli = Cli::parse();

  main_cmd(cli.command);
}


fn main_cmd(cmd: MainCommands) {
  match cmd {

    MainCommands::Init{path, name, desc, no_git} => {
      let info = init::InitInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name: name.unwrap_or(String::new()),
        desc: desc.unwrap_or(String::new()),
        no_git,
      };

      match route::init::init(info) {
        Ok(()) => (),
        Err(e) => {
          match e {
            crate::control::module::CompilerError::Str(msg) => eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), msg),
            _ => eprintln!("{}{} {:?}", "error".red().bold(), ":".bright_black(), e),
          }
          std::process::exit(1);
        }
      };
    }
    
    MainCommands::Build{path, verbose, timings, ast_dump, hir_dump} => {
      let info = build::BuildInfo {
        path: path.to_str().unwrap_or(""),
        verbose,
        timings,
        ast_dump,
        hir_dump,
        check_only: false,
      };

      match route::build::build(info) {
        Ok(()) => (),
        Err(e) => {
          match e {
            crate::control::module::CompilerError::Str(msg) => eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), msg),
            _ => eprintln!("{}{} {:?}", "error".red().bold(), ":".bright_black(), e),
          }
          std::process::exit(1);
        }
      };
    }

    MainCommands::Check{path, verbose, timings, ast_dump} => {
      let info = route::build::BuildInfo {
        path: path.to_str().unwrap_or(""),
        verbose,
        timings,
        ast_dump,
        hir_dump: false,
        check_only: true,
      };

      match route::build::build(info) {
        Ok(()) => (),
        Err(e) => {
          match e {
            crate::control::module::CompilerError::Str(msg) => eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), msg),
            _ => eprintln!("{}{} {:?}", "error".red().bold(), ":".bright_black(), e),
          }
          std::process::exit(1);
        }
      };
    }
    
    MainCommands::Deps{command} => {
      deps_cmd(command);
    }

  }
}


fn deps_cmd(cmd: DepsCommands) {
  match cmd {

    DepsCommands::Sync{path} => {
      let info = route::deps::sync::SyncInfo {
        path: path.to_str().unwrap_or("").to_string(),
      };
      match route::deps::sync::sync(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }

    DepsCommands::Add{path, name, url} => {
      let info = route::deps::add::AddInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name,
        url,
      };
      match route::deps::add::add(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }

    DepsCommands::Rm{path, name} => {
      let info = route::deps::rm::RmInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name,
      };
      match route::deps::rm::rm(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), e);
          std::process::exit(1);
        }
      }
    }
  
  }
}
