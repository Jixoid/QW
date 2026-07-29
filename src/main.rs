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
  pub command: Commands,
}


#[derive(Subcommand)]
pub enum Commands {
  Init {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    desc: Option<String>,

    #[arg(long)]
    no_git: bool,
  },

  Build {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(long)]
    timings: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long)]
    ast_dump: bool,
    #[arg(long)]
    hir_dump: bool,
  },
}


fn main() {
  let cli = Cli::parse();

  match cli.command {

    Commands::Init{path, name, desc, no_git} => {
      let info = init::InitInfo {
        path: path.to_str().unwrap_or("").to_string(),
        name: name.unwrap_or(String::new()),
        desc: desc.unwrap_or(String::new()),
        no_git,
      };

      match init::init(info) {
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
    
    Commands::Build{path, verbose, timings, ast_dump, hir_dump} => {
      let info = build::BuildInfo {
        path: path.to_str().unwrap_or(""),
        verbose,
        timings,
        ast_dump,
        hir_dump
      };

      match build::build(info) {
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

  }
}
