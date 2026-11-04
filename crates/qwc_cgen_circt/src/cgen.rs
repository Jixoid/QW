/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::path::Path;

use qwc_cgen::{GenOptions, GenOutput, IBackend, ICodeGen, ILinkObj, IRun, LinkOptions};
use qwc_mir as mir;
use qwc_string_interner::StrInterner;

use crate::{SymbLow, adapter};


pub struct GenCIRCT;

impl IBackend for GenCIRCT {
	fn code_gen(&self) -> Box<dyn ICodeGen> { Box::new(Self) }
	fn link_obj(&self) -> Box<dyn ILinkObj> { Box::new(Self) }
	fn run(&self) -> Option<Box<dyn IRun>> { None }
}

impl ICodeGen for GenCIRCT {
  fn generate(&self, mir: &mir::Krate, sin: &StrInterner, outdir: &Path, _options: &GenOptions) -> Result<GenOutput, String> {
    let mut adap = adapter::Module::default();

    let mut ctx = Ctx{
      src: mir,
      sin,
      adap: &mut adap,
    };

    for idx in 0..mir.symbols_len() {
      let id = mir::SymbId::from_any(mir::AnyId::new(idx as u32, mir::id::NodeKind::Symb));

      SymbLow::low(&mut ctx, id)?;
    }
    
    drop(ctx);
    let text = adap.finish();
    let text_clone = text.clone();

    std::fs::write(outdir.join("out.mlir"), &text).map_err(|err| format!("io: {err}"))?;

    Ok(GenOutput {
      binary: text.into_bytes(),
      text_ir: Some(text_clone),
    })
  }
}

impl ILinkObj for GenCIRCT {
  fn link(&self, _target: &Path, _out: &Path, _deps: &[&Path], _args: &[&str], _options: &LinkOptions) -> Result<(), String> {
    Ok(())
  }
}


pub struct Ctx<'mir, 'adap> {
  pub src: &'mir mir::Krate,
  #[allow(dead_code)]
  pub sin: &'mir StrInterner,
  pub adap: &'adap mut adapter::Module,
}
