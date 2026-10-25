/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use rustc_hash::FxHashMap;
use qwc_serial::{EncodeErr, SerialMapEncode};
use qwc_string_interner::{Sid, StrInterner};

pub struct StringEncoderMap<'a> {
  pub sin: &'a StrInterner,
  pub string_map: FxHashMap<Sid, u32>,
  pub strings: Vec<String>,
}

impl<'a> StringEncoderMap<'a> {
  pub fn new(sin: &'a StrInterner) -> Self {
    Self {
      sin,
      string_map: FxHashMap::default(),
      strings: Vec::new(),
    }
  }
}

impl<'a> SerialMapEncode<Sid> for StringEncoderMap<'a> {
  fn map_encode(&mut self, sid: &Sid) -> Result<u32, EncodeErr> {
    if let Some(&idx) = self.string_map.get(sid) {
      Ok(idx)
    } else {
      let idx = self.strings.len() as u32;
      let s = self.sin.str(*sid).to_string();
      self.strings.push(s);
      self.string_map.insert(*sid, idx);
      Ok(idx)
    }
  }
}

impl<'a> qwc_serial::MapEncoder for StringEncoderMap<'a> {
  fn map_encode_any(&mut self, type_id: std::any::TypeId, val: &dyn std::any::Any) -> Result<Option<u32>, EncodeErr> {
    if type_id == std::any::TypeId::of::<Sid>() {
      let sid = val.downcast_ref::<Sid>().ok_or(EncodeErr::ImpossibleData)?;
      let id = self.map_encode(sid)?;
      Ok(Some(id))
    } else {
      Ok(None)
    }
  }
}

