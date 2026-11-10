/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{io::Write, path::Path};

use qwc_mir::Krate;
use qwc_string_interner::StrInterner;


#[derive(Copy, Clone)]
pub enum Optimization { None, Less, Default, Aggressive }

#[derive(Copy, Clone)]
pub enum RelocMode { PIC, Static }

#[derive(Copy, Clone)]
pub enum CodeModel { Small, Medium, Large, Kernel }

#[derive(Copy, Clone)]
pub enum LinkTarget { Shared, Exec }



pub struct GenOutput {
  pub binary: Vec<u8>,
  pub text_ir: Option<String>,
}

pub struct GenOptions {
  pub triple: Option<String>,
  pub opt_level: Optimization,
  pub emit_ir: bool,
  
  pub reloc_mode: Option<RelocMode>,
  pub code_model: Option<CodeModel>,
}

pub struct LinkOptions {
  pub triple: Option<String>,
  
  pub lto: bool,
  pub strip: bool,

  pub target: Option<LinkTarget>,
  pub is_dynamic: bool,
  pub reloc_mode: Option<RelocMode>,
  pub code_model: Option<CodeModel>,
  pub output: Option<std::path::PathBuf>,
}

pub struct RunEnv<'a> {
  pub arg: &'a [&'a str],
  pub env: &'a [&'a str],
  pub panic: &'a dyn Fn(),
  pub stdout: &'a dyn Write,
  pub stderr: &'a dyn Write,
}


pub trait IBackend {
  fn code_gen(&self) -> Box<dyn ICodeGen>;
  fn link_obj(&self) -> Box<dyn ILinkObj>;

  fn run(&self) -> Option<Box<dyn IRun>>;
}


pub trait ICodeGen {
  fn generate(&self, mir: &Krate, sin: &StrInterner, outdir: &Path, options: &GenOptions) -> Result<GenOutput, String>;
}

pub trait ILinkObj {
  fn link(&self, target: &Path, out: &Path, deps: &[&Path], args: &[&str], options: &LinkOptions) -> Result<(), String>;
}

pub trait IRun {
  fn run(&self, target: &Path, args: &[&str]) -> Result<i32, String>;
}
