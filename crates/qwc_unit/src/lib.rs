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
use qwc_serial::{DecodeCtx, Decoder, EncodeCtx, Encoder};
use qwc_string_interner::{Sid, StrInterner};

mod codec;
use codec::StringEncoderMap;


const MAGIC: [u8; 8] = *b"\x1BQAOSQWU";
const VERSION: u32 = 1;


struct UnitLoader {
  raw: Arc<[u8]>,
  local_sids: Vec<Sid>,
  type_offsets: Vec<u32>,
  expr_offsets: Vec<u32>,
  item_offsets: Vec<u32>,
  thin_offsets: Vec<u32>,
  dpath_offsets: Vec<u32>,
}

impl hir::LazyLoader for UnitLoader {
  fn load_type(&self, idx: usize) -> hir::Type {
    let offset = self.type_offsets[idx] as usize;
    let mut slice = &self.raw[offset..];
    let mut dec = DecodeCtx::new_with_map(&mut slice, &self.local_sids);
    hir::Type::decode(&mut dec).expect("failed to decode lazy type")
  }

  fn load_expr(&self, idx: usize) -> hir::Expr {
    let offset = self.expr_offsets[idx] as usize;
    let mut slice = &self.raw[offset..];
    let mut dec = DecodeCtx::new_with_map(&mut slice, &self.local_sids);
    hir::Expr::decode(&mut dec).expect("failed to decode lazy expr")
  }

  fn load_item(&self, idx: usize) -> hir::Item {
    let offset = self.item_offsets[idx] as usize;
    let mut slice = &self.raw[offset..];
    let mut dec = DecodeCtx::new_with_map(&mut slice, &self.local_sids);
    hir::Item::decode(&mut dec).expect("failed to decode lazy item")
  }

  fn load_thing(&self, idx: usize) -> hir::Thing {
    let offset = self.thin_offsets[idx] as usize;
    let mut slice = &self.raw[offset..];
    let mut dec = DecodeCtx::new_with_map(&mut slice, &self.local_sids);
    hir::Thing::decode(&mut dec).expect("failed to decode lazy thing")
  }

  fn load_dpath(&self, idx: usize) -> hir::DefPath {
    let offset = self.dpath_offsets[idx] as usize;
    let mut slice = &self.raw[offset..];
    let mut dec = DecodeCtx::new_with_map(&mut slice, &self.local_sids);
    hir::DefPath::decode(&mut dec).expect("failed to decode lazy defpath")
  }
}


pub struct Unit;

impl Unit {
  pub fn serialize(w: &mut dyn Write, hir: &hir::Krate, exp: &ExportMap, sin: &StrInterner) -> Result<(), String> {
    let mut buf = Vec::new();
    let mut str_map = StringEncoderMap::new(sin);
    let mut enc = EncodeCtx::new_with_map(&mut buf, &mut str_map);


    // Header placeholder (49 bytes)
    MAGIC.encode(&mut enc)?;
    VERSION.encode(&mut enc)?;
    hir.cid().encode(&mut enc)?;
    hir.root().encode(&mut enc)?;

    let types_count = hir.types_count();
    let exprs_count = hir.exprs_count();
    let items_count = hir.items_count();
    let thins_count = hir.thins_count();
    let dpaths_count = hir.dpaths_count();

    (types_count as u32).encode(&mut enc)?;
    (exprs_count as u32).encode(&mut enc)?;
    (items_count as u32).encode(&mut enc)?;
    (thins_count as u32).encode(&mut enc)?;
    (dpaths_count as u32).encode(&mut enc)?;
    let footer_offset_pos = enc.written();
    0u64.encode(&mut enc)?; // placeholder for footer_offset

    // Data section: Types
    let mut type_offsets = Vec::with_capacity(types_count);
    for idx in 0..types_count {
      type_offsets.push(enc.written() as u32);
      let ty = hir.get_type_by_idx(idx);
      ty.encode(&mut enc)?;
    }

    // Data section: Exprs
    let mut expr_offsets = Vec::with_capacity(exprs_count);
    for idx in 0..exprs_count {
      expr_offsets.push(enc.written() as u32);
      let expr = hir.get_expr_by_idx(idx);
      expr.encode(&mut enc)?;
    }

    // Data section: Items
    let mut item_offsets = Vec::with_capacity(items_count);
    for idx in 0..items_count {
      item_offsets.push(enc.written() as u32);
      let item = hir.get_item_by_idx(idx);
      item.encode(&mut enc)?;
    }

    // Data section: Things
    let mut thin_offsets = Vec::with_capacity(thins_count);
    for idx in 0..thins_count {
      thin_offsets.push(enc.written() as u32);
      let thing = hir.get_thin_by_idx(idx);
      thing.encode(&mut enc)?;
    }

    // Data section: DefPaths
    let mut dpath_offsets = Vec::with_capacity(dpaths_count);
    for idx in 0..dpaths_count {
      dpath_offsets.push(enc.written() as u32);
      let dpath = hir.get_dpath_by_idx(idx);
      dpath.encode(&mut enc)?;
    }

    // ExtraData section
    let extra_offset = enc.written() as u64;
    let (extra_ids, extra_kinds) = hir.extra_raw();
    (extra_ids.len() as u32).encode(&mut enc)?;
    for (id, kind) in extra_ids.iter().zip(extra_kinds.iter()) {
      id.encode(&mut enc)?;
      kind.encode(&mut enc)?;
    }

    // ExportMap section
    let export_offset = enc.written() as u64;
    exp.encode(&mut enc)?;

    // ImplTable section
    let impl_offset = enc.written() as u64;
    let impl_items = hir.impl_items();
    impl_items.encode(&mut enc)?;

    // Drop enc so str_map can be accessed
    let written_so_far = enc.written();
    drop(enc);

    // StringTable section
    let string_table_offset = written_so_far as u64;
    let mut enc = EncodeCtx::new_with_offset(&mut buf, written_so_far);
    str_map.strings.encode(&mut enc)?;

    // IndexTable section
    let index_table_offset = enc.written() as u64;
    for o in &type_offsets { o.encode(&mut enc)?; }
    for o in &expr_offsets { o.encode(&mut enc)?; }
    for o in &item_offsets { o.encode(&mut enc)?; }
    for o in &thin_offsets { o.encode(&mut enc)?; }
    for o in &dpath_offsets { o.encode(&mut enc)?; }

    // Footer section
    let footer_offset = enc.written() as u64;
    string_table_offset.encode(&mut enc)?;
    index_table_offset.encode(&mut enc)?;
    export_offset.encode(&mut enc)?;
    impl_offset.encode(&mut enc)?;
    extra_offset.encode(&mut enc)?;
    drop(enc);

    // Patch footer_offset in header
    LittleEndian::write_u64(&mut buf[footer_offset_pos..footer_offset_pos + 8], footer_offset);

    // Write to stream
    w.write_all(&buf).map_err(|e| format!("io write error: {e}"))?;
    Ok(())
  }

  pub fn deserialize(r: &mut dyn Read, sin: &mut StrInterner) -> Result<(hir::Krate, ExportMap), String> {
    let mut buffer = Vec::new();
    r.read_to_end(&mut buffer).map_err(|e| format!("io read error: {e}"))?;

    let mut slice = &buffer[..];
    let mut dec = DecodeCtx::new(&mut slice);

    let magic = <[u8; 8]>::decode(&mut dec)?;
    if magic != MAGIC { return Err(String::from("corrupt qwu: invalid magic")); }

    let version = u32::decode(&mut dec)?;
    if version != VERSION {
      return Err(format!("unsupported qwu version: {version}"));
    }

    let cid = hir::CID::decode(&mut dec)?;
    let root = Option::<hir::ItemId>::decode(&mut dec)?;

    let types_count = u32::decode(&mut dec)? as usize;
    let exprs_count = u32::decode(&mut dec)? as usize;
    let items_count = u32::decode(&mut dec)? as usize;
    let thins_count = u32::decode(&mut dec)? as usize;
    let dpaths_count = u32::decode(&mut dec)? as usize;
    let footer_offset = u64::decode(&mut dec)? as usize;

    if footer_offset + 40 > buffer.len() {
      return Err(String::from("corrupt qwu: invalid footer offset"));
    }

    let mut footer_slice = &buffer[footer_offset..];
    let mut fdec = DecodeCtx::new(&mut footer_slice);
    let string_table_offset = u64::decode(&mut fdec)? as usize;
    let index_table_offset = u64::decode(&mut fdec)? as usize;
    let export_offset = u64::decode(&mut fdec)? as usize;
    let impl_offset = u64::decode(&mut fdec)? as usize;
    let extra_offset = u64::decode(&mut fdec)? as usize;

    // Decode String Table and intern all strings into target compiler
    let mut slice = &buffer[string_table_offset..];
    let mut str_dec = DecodeCtx::new(&mut slice);
    let strings = Vec::<String>::decode(&mut str_dec).map_err(|e| format!("string table decode error: {e}"))?;
    let local_sids: Vec<Sid> = strings.into_iter().map(|s| sin.sid(&s)).collect();

    // Decode Index Tables
    let mut slice = &buffer[index_table_offset..];
    let mut idx_dec = DecodeCtx::new(&mut slice);
    let mut type_offsets = Vec::with_capacity(types_count);
    for _ in 0..types_count { type_offsets.push(u32::decode(&mut idx_dec)?); }
    let mut expr_offsets = Vec::with_capacity(exprs_count);
    for _ in 0..exprs_count { expr_offsets.push(u32::decode(&mut idx_dec)?); }
    let mut item_offsets = Vec::with_capacity(items_count);
    for _ in 0..items_count { item_offsets.push(u32::decode(&mut idx_dec)?); }
    let mut thin_offsets = Vec::with_capacity(thins_count);
    for _ in 0..thins_count { thin_offsets.push(u32::decode(&mut idx_dec)?); }
    let mut dpath_offsets = Vec::with_capacity(dpaths_count);
    for _ in 0..dpaths_count { dpath_offsets.push(u32::decode(&mut idx_dec)?); }

    // Decode ExtraData
    let mut slice = &buffer[extra_offset..];
    let mut ext_dec = DecodeCtx::new(&mut slice);
    let extra_cnt = u32::decode(&mut ext_dec)? as usize;
    let mut extra_ids = Vec::with_capacity(extra_cnt);
    let mut extra_kinds = Vec::with_capacity(extra_cnt);
    for _ in 0..extra_cnt {
      extra_ids.push(hir::HirId::<hir::SpecAny>::decode(&mut ext_dec)?);
      extra_kinds.push(hir::NodeKind::decode(&mut ext_dec)?);
    }

    // Decode Impl Table
    let mut slice = &buffer[impl_offset..];
    let mut imp_dec = DecodeCtx::new(&mut slice);
    let impl_items = Vec::<hir::ItemId>::decode(&mut imp_dec)?;

    // Wrap raw buffer and create Lazy Krate
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

    
    let krate = hir::Krate::new_lazy(
      cid,
      root,
      loader,
      (types_count, exprs_count, items_count, thins_count, dpaths_count),
      (extra_ids, extra_kinds),
      impl_items,
    );

    // Decode ExportMap
    let mut slice = &raw[export_offset..];
    let mut exp_dec = DecodeCtx::new_with_map(&mut slice, &local_sids);
    let exp = ExportMap::decode(&mut exp_dec).map_err(|e| format!("export map decode error: {e}"))?;


    Ok((krate, exp))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;
  use qwc_hir::PushApi;

  #[test]
  fn test_unit_roundtrip_none_root() {
    let mut sin = StrInterner::default();
    let cid = hir::Deps::default().get_next_id();
    let krate = hir::Krate::new(cid);
    let exp = ExportMap::new(cid, None);

    let mut buf = Vec::new();
    Unit::serialize(&mut buf, &krate, &exp, &sin).unwrap();

    let mut cursor = Cursor::new(buf);
    let (loaded_krate, _loaded_exp) = Unit::deserialize(&mut cursor, &mut sin).unwrap();

    assert_eq!(loaded_krate.cid(), cid);
    assert_eq!(loaded_krate.root(), None);
  }

  #[test]
  fn test_unit_roundtrip_some_root() {
    let mut sin = StrInterner::default();
    let cid = hir::Deps::default().get_next_id();
    let mut krate = hir::Krate::new(cid);
    let sid = sin.sid("test_item");
    let path = hir::DefPath::push(&mut krate, hir::DefPath::Root(sid));
    let item = hir::Item {
      vis: hir::ItemVis::Private,
      name: Some(sid),
      kind: hir::ItemKind::RootNS { rng: hir::ItemRng::empty(cid) },
      attr: hir::ItemAttr::empty(),
      path,
    };
    let item_id = hir::Item::push(&mut krate, item);
    krate.set_root(item_id);

    let exp = ExportMap::new(cid, Some(item_id));

    let mut buf = Vec::new();
    Unit::serialize(&mut buf, &krate, &exp, &sin).unwrap();

    let mut cursor = Cursor::new(buf);
    let (loaded_krate, _loaded_exp) = Unit::deserialize(&mut cursor, &mut sin).unwrap();

    assert_eq!(loaded_krate.cid(), cid);
    assert_eq!(loaded_krate.root(), Some(item_id));
    assert_eq!(loaded_krate.get(item_id).name, Some(sid));
  }
}

