/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use std::io::{Read, Write};
use std::sync::Arc;
use byteorder::{ByteOrder, LittleEndian};

use qwc_hir as hir;
use qwc_resolve::ExportMap;
use qwc_string_interner::StrInterner;

mod codec;
use codec::{Decoder, Encoder};


const MAGIC: [u8; 8] = *b"\x1BQAOSQWU";
const VERSION: u32 = 1;


struct UnitLoader {
  raw: Arc<[u8]>,
  local_sids: Vec<qwc_string_interner::Sid>,
  type_offsets: Vec<u32>,
  expr_offsets: Vec<u32>,
  item_offsets: Vec<u32>,
  thin_offsets: Vec<u32>,
  dpath_offsets: Vec<u32>,
}

impl hir::LazyLoader for UnitLoader {
  fn load_type(&self, idx: usize) -> hir::Type {
    let offset = self.type_offsets[idx] as usize;
    let mut dec = Decoder::new(&self.raw[offset..], &self.local_sids);
    dec.read_type()
  }

  fn load_expr(&self, idx: usize) -> hir::Expr {
    let offset = self.expr_offsets[idx] as usize;
    let mut dec = Decoder::new(&self.raw[offset..], &self.local_sids);
    dec.read_expr()
  }

  fn load_item(&self, idx: usize) -> hir::Item {
    let offset = self.item_offsets[idx] as usize;
    let mut dec = Decoder::new(&self.raw[offset..], &self.local_sids);
    dec.read_item()
  }

  fn load_thing(&self, idx: usize) -> hir::Thing {
    let offset = self.thin_offsets[idx] as usize;
    let mut dec = Decoder::new(&self.raw[offset..], &self.local_sids);
    dec.read_thing()
  }

  fn load_dpath(&self, idx: usize) -> hir::DefPath {
    let offset = self.dpath_offsets[idx] as usize;
    let mut dec = Decoder::new(&self.raw[offset..], &self.local_sids);
    dec.read_defpath()
  }
}


pub struct Unit;

impl Unit {
  pub fn serialize(w: &mut dyn Write, hir: &hir::Krate, exp: &ExportMap, sin: &StrInterner) -> Result<(), String> {
    let mut enc = Encoder::new(sin);

    // 1. Header placeholder (46 bytes)
    enc.buf.extend_from_slice(&MAGIC); // 8 bytes
    enc.write_u32(VERSION);             // 4 bytes
    enc.write_cid(hir.cid());           // 2 bytes
    let root_raw = match hir.root() {
      Some(id) => id.idx() + 1,
      None => 0,
    };
    enc.write_u32(root_raw);            // 4 bytes

    let types_count = hir.types_count();
    let exprs_count = hir.exprs_count();
    let items_count = hir.items_count();
    let thins_count = hir.thins_count();
    let dpaths_count = hir.dpaths_count();

    enc.write_u32(types_count as u32);
    enc.write_u32(exprs_count as u32);
    enc.write_u32(items_count as u32);
    enc.write_u32(thins_count as u32);
    enc.write_u32(dpaths_count as u32); // 20 bytes total for counts
    enc.write_u64(0);                   // 8 bytes placeholder for footer_offset

    debug_assert_eq!(enc.buf.len(), 46);

    // 2. Data section: Types
    let mut type_offsets = Vec::with_capacity(types_count);
    for idx in 0..types_count {
      type_offsets.push(enc.buf.len() as u32);
      let ty = hir.get_type_by_idx(idx);
      enc.write_type(ty);
    }

    // 3. Data section: Exprs
    let mut expr_offsets = Vec::with_capacity(exprs_count);
    for idx in 0..exprs_count {
      expr_offsets.push(enc.buf.len() as u32);
      let expr = hir.get_expr_by_idx(idx);
      enc.write_expr(expr);
    }

    // 4. Data section: Items
    let mut item_offsets = Vec::with_capacity(items_count);
    for idx in 0..items_count {
      item_offsets.push(enc.buf.len() as u32);
      let item = hir.get_item_by_idx(idx);
      enc.write_item(item);
    }

    // 5. Data section: Things
    let mut thin_offsets = Vec::with_capacity(thins_count);
    for idx in 0..thins_count {
      thin_offsets.push(enc.buf.len() as u32);
      let thing = hir.get_thin_by_idx(idx);
      enc.write_thing(thing);
    }

    // 6. Data section: DefPaths
    let mut dpath_offsets = Vec::with_capacity(dpaths_count);
    for idx in 0..dpaths_count {
      dpath_offsets.push(enc.buf.len() as u32);
      let dpath = hir.get_dpath_by_idx(idx);
      enc.write_defpath(dpath);
    }

    // 7. ExtraData section
    let extra_offset = enc.buf.len() as u64;
    let (extra_ids, extra_kinds) = hir.extra_raw();
    enc.write_u32(extra_ids.len() as u32);
    for (id, kind) in extra_ids.iter().zip(extra_kinds.iter()) {
      enc.write_hir_id(*id);
      enc.write_u8(match kind {
        hir::NodeKind::Any => 0,
        hir::NodeKind::Type => 1,
        hir::NodeKind::Expr => 2,
        hir::NodeKind::Item => 3,
        hir::NodeKind::Thing => 4,
        hir::NodeKind::DefPath => 5,
      });
    }

    // 8. ExportMap section
    let export_offset = enc.buf.len() as u64;
    enc.write_export_map(exp);

    // 9. ImplTable section
    let impl_offset = enc.buf.len() as u64;
    let impl_items = hir.impl_items();
    enc.write_u32(impl_items.len() as u32);
    for id in &impl_items {
      enc.write_hir_id(*id);
    }

    // 10. StringTable section
    let string_table_offset = enc.buf.len() as u64;
    enc.write_u32(enc.strings.len() as u32);
    for s in &enc.strings.clone() {
      enc.write_str(s);
    }

    // 11. IndexTable section
    let index_table_offset = enc.buf.len() as u64;
    for o in &type_offsets { enc.write_u32(*o); }
    for o in &expr_offsets { enc.write_u32(*o); }
    for o in &item_offsets { enc.write_u32(*o); }
    for o in &thin_offsets { enc.write_u32(*o); }
    for o in &dpath_offsets { enc.write_u32(*o); }

    // 12. Footer section
    let footer_offset = enc.buf.len() as u64;
    enc.write_u64(string_table_offset);
    enc.write_u64(index_table_offset);
    enc.write_u64(export_offset);
    enc.write_u64(impl_offset);
    enc.write_u64(extra_offset);

    // Patch footer_offset in header at bytes 38..46
    LittleEndian::write_u64(&mut enc.buf[38..46], footer_offset);

    // Write to stream
    w.write_all(&enc.buf).map_err(|e| format!("io write error: {e}"))?;
    Ok(())
  }

  pub fn deserialize(r: &mut dyn Read, sin: &mut StrInterner) -> Result<(hir::Krate, ExportMap), String> {
    let mut buffer = Vec::new();
    r.read_to_end(&mut buffer).map_err(|e| format!("io read error: {e}"))?;

    if buffer.len() < 46 {
      return Err(String::from("qwu file too short"));
    }

    if buffer[0..8] != MAGIC {
      return Err(String::from("corrupt qwu: invalid magic"));
    }

    let version = LittleEndian::read_u32(&buffer[8..12]);
    if version != VERSION {
      return Err(format!("unsupported qwu version: {version}"));
    }

    let cid_raw = LittleEndian::read_u16(&buffer[12..14]);
    let cid = hir::CID::from_raw(cid_raw);
    let root_raw = LittleEndian::read_u32(&buffer[14..18]);

    let types_count = LittleEndian::read_u32(&buffer[18..22]) as usize;
    let exprs_count = LittleEndian::read_u32(&buffer[22..26]) as usize;
    let items_count = LittleEndian::read_u32(&buffer[26..30]) as usize;
    let thins_count = LittleEndian::read_u32(&buffer[30..34]) as usize;
    let dpaths_count = LittleEndian::read_u32(&buffer[34..38]) as usize;
    let footer_offset = LittleEndian::read_u64(&buffer[38..46]) as usize;

    if footer_offset + 40 > buffer.len() {
      return Err(String::from("corrupt qwu: invalid footer offset"));
    }

    let string_table_offset = LittleEndian::read_u64(&buffer[footer_offset..footer_offset + 8]) as usize;
    let index_table_offset = LittleEndian::read_u64(&buffer[footer_offset + 8..footer_offset + 16]) as usize;
    let export_offset = LittleEndian::read_u64(&buffer[footer_offset + 16..footer_offset + 24]) as usize;
    let impl_offset = LittleEndian::read_u64(&buffer[footer_offset + 24..footer_offset + 32]) as usize;
    let extra_offset = LittleEndian::read_u64(&buffer[footer_offset + 32..footer_offset + 40]) as usize;

    // 1. Decode String Table and intern all strings into target compiler
    let mut str_dec = Decoder::new(&buffer[string_table_offset..], &[]);
    let str_count = str_dec.read_u32() as usize;
    let mut local_sids = Vec::with_capacity(str_count);
    for _ in 0..str_count {
      let s = str_dec.read_str();
      local_sids.push(sin.sid(&s));
    }

    // 2. Decode Index Tables
    let mut idx_dec = Decoder::new(&buffer[index_table_offset..], &local_sids);
    let mut type_offsets = Vec::with_capacity(types_count);
    for _ in 0..types_count { type_offsets.push(idx_dec.read_u32()); }
    let mut expr_offsets = Vec::with_capacity(exprs_count);
    for _ in 0..exprs_count { expr_offsets.push(idx_dec.read_u32()); }
    let mut item_offsets = Vec::with_capacity(items_count);
    for _ in 0..items_count { item_offsets.push(idx_dec.read_u32()); }
    let mut thin_offsets = Vec::with_capacity(thins_count);
    for _ in 0..thins_count { thin_offsets.push(idx_dec.read_u32()); }
    let mut dpath_offsets = Vec::with_capacity(dpaths_count);
    for _ in 0..dpaths_count { dpath_offsets.push(idx_dec.read_u32()); }

    // 3. Decode ExtraData
    let mut ext_dec = Decoder::new(&buffer[extra_offset..], &local_sids);
    let extra_cnt = ext_dec.read_u32() as usize;
    let mut extra_ids = Vec::with_capacity(extra_cnt);
    let mut extra_kinds = Vec::with_capacity(extra_cnt);
    for _ in 0..extra_cnt {
      extra_ids.push(ext_dec.read_hir_id::<hir::SpecAny>());
      let kind = match ext_dec.read_u8() {
        0 => hir::NodeKind::Any,
        1 => hir::NodeKind::Type,
        2 => hir::NodeKind::Expr,
        3 => hir::NodeKind::Item,
        4 => hir::NodeKind::Thing,
        _ => hir::NodeKind::DefPath,
      };
      extra_kinds.push(kind);
    }

    // 4. Decode Impl Table
    let mut imp_dec = Decoder::new(&buffer[impl_offset..], &local_sids);
    let impl_cnt = imp_dec.read_u32() as usize;
    let mut impl_items = Vec::with_capacity(impl_cnt);
    for _ in 0..impl_cnt {
      impl_items.push(imp_dec.read_hir_id::<hir::Item>());
    }

    // 5. Wrap raw buffer and create Lazy Krate
    let raw: Arc<[u8]> = Arc::from(buffer.into_boxed_slice());
    let loader = Arc::new(UnitLoader {
      raw: raw.clone(),
      local_sids: local_sids.clone(),
      type_offsets,
      expr_offsets,
      item_offsets,
      thin_offsets,
      dpath_offsets,
    });

    let root_id = if root_raw > 0 {
      Some(hir::ItemId::new(cid, root_raw - 1))
    } else {
      None
    };

    let krate = hir::Krate::new_lazy(
      cid,
      root_id,
      loader,
      (types_count, exprs_count, items_count, thins_count, dpaths_count),
      (extra_ids, extra_kinds),
      impl_items,
    );

    // 6. Decode ExportMap
    let mut exp_dec = Decoder::new(&raw[export_offset..], &local_sids);
    let exp = exp_dec.read_export_map();

    Ok((krate, exp))
  }
}
