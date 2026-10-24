/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::io::{Read, Write};
use byteorder::{ReadBytesExt, LittleEndian};

use qwc_hir as hir;
use qwc_resolve::ExportMap;


pub struct Unit;

impl Unit {
  pub fn serialize(w: &mut dyn Write, hir: &hir::Krate, exp: &ExportMap) -> Result<(), String> {
    w.write_all(&MAGIC).map_err(|err| format!("io: {err}"))?;
    w.write_all(&SUBMAGIC).map_err(|err| format!("io: {err}"))?;

    let a = postcard::to_allocvec(hir).map_err(|err| format!("serialize: {err}"))?;
    w.write_all(&(a.len() as u64).to_le_bytes()).map_err(|err| format!("io: {err}"))?;
    w.write_all(&a).map_err(|err| format!("io: {err}"))?;
    
    let a = postcard::to_allocvec(exp).map_err(|err| format!("serialize: {err}"))?;
    w.write_all(&(a.len() as u64).to_le_bytes()).map_err(|err| format!("io: {err}"))?;
    w.write_all(&a).map_err(|err| format!("io: {err}"))?;

    Ok(())
  }

  pub fn deserialize(r: &mut dyn Read) -> Result<(hir::Krate, ExportMap), String> {
    let mut magic_buf = [0_u8; 22];
    
    r.read_exact(&mut magic_buf[0..10]).map_err(|err| format!("io: {err}"))?; if magic_buf[0..10] != MAGIC    { return Err(format!("file is corrupt1")) };
    r.read_exact(&mut magic_buf[0..22]).map_err(|err| format!("io: {err}"))?; if magic_buf[0..22] != SUBMAGIC { return Err(format!("file is corrupt2")) };
    
    let s = r.read_u64::<LittleEndian>().map_err(|err| format!("io: {err}"))?;
    let mut v = vec![0_u8; s as usize];
    r.read_exact(&mut v).map_err(|err| format!("io: {err}"))?;
    let hir = postcard::from_bytes::<hir::Krate>(&v).map_err(|err| format!("deserialize: {err}"))?;
    
    let s = r.read_u64::<LittleEndian>().map_err(|err| format!("io: {err}"))?;
    let mut v = vec![0_u8; s as usize];
    r.read_exact(&mut v).map_err(|err| format!("io: {err}"))?;
    let exp = postcard::from_bytes::<ExportMap>(&v).map_err(|err| format!("deserialize: {err}"))?;

    Ok((hir, exp))
  }
}


const MAGIC: [u8; 10] = *b"\x1B\xC2\xABQAOS!\xC2\xBB";
const SUBMAGIC: [u8; 22] = *b"qw-compiled-crate-////";

const _VERSION: &str = env!("CARGO_PKG_VERSION");
