/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

mod error;
mod build;
mod conf;
mod init;
mod run;
//mod deps;
//mod pkg;

use error::Error;
use conf::parse_conf;


pub fn route() {
  main_cmd(Cli::parse().command);
}


#[derive(Parser)]
#[command(name = "qw", version, about = "qw compiler")]
struct Cli {
  #[command(subcommand)]
  pub command: MainCommands,
}

#[derive(Subcommand)]
enum MainCommands {
  /// Create a new qw package
  #[command(alias = "i")]
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

    /// Force init overwrite
    #[arg(short, long)]
    force: bool,
  },

  #[command(alias = "r")]
  Run {
    /// Directory to build
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Display execution times for each compiler phase
    #[arg(long)]
    timings: bool,
  
    /// Display memory storage for each compiler phase
    #[arg(long)]
    usages: bool,

    /// Use verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long, value_delimiter = ',', value_enum)]
    dump: Vec<DumpStage>,
  },

  #[command(alias = "b")]
  /// Compile the package
  Build {
    /// Directory to build
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Target build destination
    #[arg(long, default_value = "debug")]
    variant: BuildVariant,

    /// Target start routine
    #[arg(long, default_value = "cqrt")]
    start_routine: BuildStartRoutine,

    /// Target triple
    #[arg(long)]
    triple: Option<String>,
    
    /// Source RTL
    #[arg(long)]
    rtl: Option<String>,

    /// Display execution times for each compiler phase
    #[arg(long)]
    timings: bool,

    /// Display memory storage for each compiler phase
    #[arg(long)]
    usages: bool,

    /// Use verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    #[arg(long, value_delimiter = ',', value_enum)]
    dump: Vec<DumpStage>,
  },

  #[command(alias = "c")]
  /// Check the package for errors
  Check {
    /// Directory to check
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Display execution times for each compiler phase
    #[arg(long)]
    timings: bool,

    /// Display memory storage for each compiler phase
    #[arg(long)]
    usages: bool,

    /// Use verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long, value_delimiter = ',', value_enum)]
    dump: Vec<DumpStage>,
  },

  #[cfg(feature = "lsp")]
  /// Start the language server
  Lsp,

  /*
  /// Manage project dependencies
  Deps {
    #[command(subcommand)]
    command: deps::DepsCommands,
  },
  
  /// Project package tool
  Pkg {
    #[command(subcommand)]
    command: pkg::PkgCommands,
  },
  */
}


fn main_cmd(cmd: MainCommands) {
  match cmd {

    MainCommands::Init{path, name, desc, no_git, force} => {
      let info = init::InitInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name: name.unwrap_or(String::new()),
        desc: desc.unwrap_or(String::new()),
        no_git,
        force,
      };

      match init::init(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }
    
    MainCommands::Run{path, timings, usages, verbose, dump} => {
      let info = run::RunInfo {
        path: &path,
        verbose,
        timings,
        usages,
        dump,
      };

      match run::run(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }

    MainCommands::Build{path, variant, start_routine, triple, rtl, verbose, timings, usages, dump} => {
      let info = build::BuildInfo {
        path: &path,
        variant,
        start_routine,
        triple,
        rtl,
        verbose,
        timings,
        usages,
        dump,
        check_only: false,
        execute: false,
      };

      match build::build(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }

    MainCommands::Check{path, verbose, timings, usages, dump} => {
      let info = build::BuildInfo {
        path: &path,
        variant: BuildVariant::Debug,
        start_routine: BuildStartRoutine::CQRT,
        triple: None,
        rtl: None,
        verbose,
        timings,
        usages,
        dump,
        check_only: true,
        execute: false,
      };

      match build::build(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }
    
    #[cfg(feature = "lsp")]
    MainCommands::Lsp => {
      match qwc_lsp::start() {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }

    //MainCommands::Deps {command} => { deps::deps_cmd(command); }
    //MainCommands::Pkg  {command} => { pkg::pkg_cmd(command); }
  }
}


#[derive(ValueEnum, Clone, Debug)]
enum BuildVariant {
  Debug,
  Release,
  RelWithDebInfo,
}


#[derive(ValueEnum, Clone, Debug)]
enum BuildStartRoutine {
  CRT,
  QRT,
  CQRT,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "lowercase")]
enum DumpStage {
  Ast,
  Scope,
  Hir,
  Export,
  Mir,
  Lir,
}
