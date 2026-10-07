/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::io::Write;

use qwc_hir as hir;
use qwc_resolve::ExportMap;


pub struct Unit;

impl Unit {
  pub fn serialize(w: &mut dyn Write, hir_cre: &hir::Krate, exp: &ExportMap) -> Result<(), String> {
    w.write_all(&MAGIC).map_err(|err|  format!("io: {}", err))?;
    w.write_all(&SUBMAGIC).map_err(|err|  format!("io: {}", err))?;

    let a = postcard::to_allocvec(hir_cre).map_err(|err| format!("serialize: {}", err))?;
    w.write_all(&a).map_err(|err| format!("io: {}", err))?;

    let a = postcard::to_allocvec(exp).map_err(|err| format!("serialize: {}", err))?;
    w.write_all(&a).map_err(|err| format!("io: {}", err))?;

    Ok(())
  }
}


pub const MAGIC: [u8; 10] = *b"\x1B\xC2\xABQAOS!\xC2\xBB";
pub const SUBMAGIC: [u8; 22] = *b"qw-compiled-create-///";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
