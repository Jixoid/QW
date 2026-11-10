/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::PathBuf;
use qwc_arena::Files;

use clap::{Parser, Subcommand, ValueEnum};
use owo_colors::OwoColorize;

mod error;
mod build;
mod clean;
mod compile;
mod init;
mod run;
//mod deps;
//mod pkg;

use error::Error;
use qwc_target::{Abi, Arch, Sys, Triple, Vendor, get_dialect};


pub fn route() {
  let args: Vec<String> = std::env::args().map(|a| {
    if a == "-exec" {
      "--exec".to_string()
    } else {
      a
    }
  }).collect();
  main_cmd(Cli::parse_from(args).command);
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

  /// Compile a single crate directly
  Compile {
    /// Directory or file to compile
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Output directory
    #[arg(long)]
    outdir: Option<PathBuf>,

    /// Output file path
    #[arg(short = 'o', long = "out")]
    out: Option<PathBuf>,

    /// External dependencies (format: NAME=PATH_TO_QWU or NAME=PATH_TO_QWU:PATH_TO_OBJ)
    #[arg(long = "dep", value_name = "NAME=QWU")]
    deps: Vec<String>,

    /// Optimization Level
    #[arg(short = 'O', default_value = "O0", default_missing_value = "1", num_args = 0..=1)]
    opt_level: OptLevel,

    /// Target build destination
    #[arg(long, default_value = "debug")]
    variant: BuildVariant,

    /// Select start routine (only executable)
    #[arg(long, default_value = "cqrt")]
    start_routine: BuildStartRoutine,
    
    /// Target machine arch
    #[arg(long)]
    arch: Option<String>,

    /// RTL Source
    #[arg(long)]
    rtl: Option<String>,

    /// PIC or Static
    #[arg(long, default_value = "pic")]
    reloc: RelocMode,

    /// Code Model
    #[arg(long, default_value = "small")]
    mcmodel: CodeModel,

    /// Link output type (executable or shared library)
    #[arg(long, value_enum, default_value = "exec")]
    link: LinkMode,

    /// Execute after build
    #[arg(long, short = 'e')]
    exec: bool,

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

  /// Compile the package
  Build {
    /// Directory to build
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Optimization Level
    #[arg(short = 'O', default_value = "O0", default_missing_value = "1", num_args = 0..=1)]
    opt_level: OptLevel,

    /// Target build destination
    #[arg(long, default_value = "debug")]
    variant: BuildVariant,

    /// Select start routine (only executable)
    #[arg(long, default_value = "cqrt")]
    start_routine: BuildStartRoutine,

    /// RTL Source
    #[arg(long)]
    rtl: Option<String>,

    /// PIC or Static
    #[arg(long, default_value = "pic")]
    reloc: RelocMode,

    /// Code Model
    #[arg(long, default_value = "small")]
    mcmodel: CodeModel,

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

  /// Clean build dir
  Clean {
    /// Directory to build
    #[arg(long, default_value = ".")]
    path: PathBuf,
  },

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

  /// Explain error code with example
  Explain {
    /// Error code
    code: String,
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

    MainCommands::Compile{path, outdir, out, deps, opt_level, variant, start_routine, arch, rtl, reloc, mcmodel, link, verbose, exec, timings, usages, dump} => {
      let variant_dir = match variant {
        BuildVariant::Debug => "debug",
        BuildVariant::Release => "release",
        BuildVariant::RelWithDebInfo => "relwithdebinfo",
      };

      if let Some(outfile) = &out {
        if let Err(err) = compile::validate_outfile(outfile, link) {
          eprintln!("{}{} {}", "error".red().bold(), ":".bright_black(), err);
          std::process::exit(1);
        }
      }

      let outdir = outdir.unwrap_or_else(|| {
        let base = if path.is_file() {
          path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
        } else {
          path.clone()
        };
        base.join("build").join(variant_dir)
      });

      let mut dep_artifacts = Vec::new();
      for dep_str in deps {
        if let Some((name, path)) = dep_str.split_once('=') {
          dep_artifacts.push(compile::DepArtifact {
            name: name.to_string(),
            path: PathBuf::from(path),
          });
        }
      }

      let arch = if let Some(arch) = arch {
        match arch.as_str() {
          "circt" => Arch::CIRCT,
          "spirv" => Arch::SpirV64,

          c @_ => {eprintln!("unknown arch: {c}"); std::process::exit(1)}
        }
      } else {
        Arch::RiscV64
      };
      
      let triple = Triple(arch, Vendor::Unknown, Sys::Linux, Abi::Unknown);
      let dialect = get_dialect(triple);

      let execute = if link == LinkMode::Shared && exec {
        eprintln!("{}{} {}", "warning".yellow().bold(), ":".bright_black(), "cannot execute a shared library; --exec is ignored");
        false
      } else {
        exec
      };

      let info = compile::CompileInfo {
        path: &path,
        outdir,
        outfile: out,
        dialect,
        opt_level,
        variant,
        start_routine,
        triple,
        rtl,
        reloc,
        mcmodel,
        link,
        verbose,
        timings,
        usages,
        dump,
        check_only: false,
        execute,
        dolink: true,
        deps: dep_artifacts,
        krate_name: qwc_config::read_conf(&path, &mut Files::default()).ok().map(|c| c.name),
      };

      match compile::compile(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }

    MainCommands::Build{path, opt_level, variant, start_routine, rtl, reloc, mcmodel, verbose, timings, usages, dump} => {
      let info = build::BuildInfo {
        path: &path,
        opt_level,
        variant,
        start_routine,
        rtl,
        reloc,
        mcmodel,
        verbose,
        timings,
        usages,
        dump,
        check_only: false,
        execute: false,
        dolink: true,
      };

      match build::build(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }

    MainCommands::Clean{path} => {
      let info = clean::CleanInfo {
        path: &path,
      };

      match clean::clean(info) {
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
        opt_level: OptLevel::O0,
        variant: BuildVariant::Debug,
        start_routine: BuildStartRoutine::CQRT,
        rtl: None,
        reloc: RelocMode::PIC,
        mcmodel: CodeModel::Small,
        verbose,
        timings,
        usages,
        dump,
        check_only: true,
        execute: false,
        dolink: false,
      };

      match build::build(info) {
        Ok(()) => (),
        Err(e) => {
          eprintln!("{}", e);
          std::process::exit(1);
        }
      };
    }
    
    MainCommands::Explain{code} => {
      qwc_explain::explain(code);
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



#[derive(ValueEnum, Debug, Clone, PartialEq)]
#[value(rename_all = "uppercase")]
pub enum OptLevel { O0, O1, O2, O3 }


#[derive(ValueEnum, Clone, Debug)]
enum BuildVariant {
  Debug,
  Release,
  RelWithDebInfo,
}


#[allow(clippy::upper_case_acronyms)]
#[derive(ValueEnum, Clone, Debug)]
enum BuildStartRoutine {
  CRT,
  QRT,
  CQRT,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum RelocMode {
  PIC,
  Static,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkMode {
  Exec,
  Shared,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum CodeModel {
  Small,
  Medium,
  Large,
  Kernel,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
enum DumpStage {
  Ast,
  Scope,
  Hir,
  Export,
  Mir,
  Lir,
}
