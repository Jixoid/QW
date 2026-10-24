/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use core::slice;
use std::{fmt, mem, ops::Range};

use serde::{Deserialize, Deserializer, de::{self, MapAccess, SeqAccess, Visitor}, ser::{Serialize, SerializeStruct, Serializer}};

const PAGE_SIZE: usize = 1024 * 1024; // 1 MB


pub struct Arena<T>
  where T: Copy
{
	data: Vec<Vec<T>>,
  len: usize,
}

impl<T: Copy> Serialize for Arena<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Arena", 2)?;
    state.serialize_field("len", &self.len)?;

    struct RawBytesHelper<'a, T: Copy>(&'a [Vec<T>], usize);

    impl<'a, T: Copy> Serialize for RawBytesHelper<'a, T> {
      fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
      where
        S: Serializer,
      {
        let total_bytes = self.1 * size_of::<T>();
        
        let mut bytes = Vec::with_capacity(total_bytes);
        for chunk in self.0 {
          let chunk_byte_len = chunk.len() * size_of::<T>();
          if chunk_byte_len == 0 { continue }

          let byte_slice = unsafe {
            slice::from_raw_parts(
              chunk.as_ptr() as *const u8,
              chunk_byte_len,
            )
          };
          bytes.extend_from_slice(byte_slice);
        }

        s.serialize_bytes(&bytes)
      }
    }

    state.serialize_field("bytes", &RawBytesHelper(&self.data, self.len))?;
    state.end()
  }
}

impl<'de, T: Copy> Deserialize<'de> for Arena<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    enum Field { Len, Bytes, Ignore }

    struct FieldVisitor;
    impl<'de> Visitor<'de> for FieldVisitor {
      type Value = Field;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`len` or `bytes`")
      }

      fn visit_str<E: de::Error>(self, value: &str) -> Result<Field, E> {
        match value {
          "len" => Ok(Field::Len),
          "bytes" => Ok(Field::Bytes),
          _ => Ok(Field::Ignore),
        }
      }

      fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Field, E> {
        match value {
          b"len" => Ok(Field::Len),
          b"bytes" => Ok(Field::Bytes),
          _ => Ok(Field::Ignore),
        }
      }
    }

    impl<'de> Deserialize<'de> for Field {
      #[inline]
      fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
      where
        D: Deserializer<'de>,
      {
        deserializer.deserialize_identifier(FieldVisitor)
      }
    }

    struct DeserializeBytesHelper(Vec<u8>);

    impl<'de> Deserialize<'de> for DeserializeBytesHelper {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
      {
        struct RawBytesVisitor;
        impl<'de> Visitor<'de> for RawBytesVisitor {
          type Value = Vec<u8>;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("byte array")
          }

          fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
            Ok(v.to_vec())
          }

          fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
            Ok(v)
          }

          fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut bytes = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(b) = seq.next_element()? {
              bytes.push(b);
            }
            Ok(bytes)
          }
        }
        deserializer.deserialize_byte_buf(RawBytesVisitor).map(DeserializeBytesHelper)
      }
    }

    struct ArenaVisitor<T>(std::marker::PhantomData<T>);
    
    impl<'de, T: Copy> Visitor<'de> for ArenaVisitor<T> {
      type Value = Arena<T>;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Arena")
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<Arena<T>, V::Error>
      where
        V: SeqAccess<'de>,
      {
        let len = seq.next_element()?
          .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        
        let bytes_helper: DeserializeBytesHelper = seq.next_element()?
          .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        reconstruct_arena(len, bytes_helper.0)
      }

      fn visit_map<V>(self, mut map: V) -> Result<Arena<T>, V::Error>
      where
        V: MapAccess<'de>,
      {
        let mut len = None;
        let mut bytes = None;

        while let Some(key) = map.next_key::<Field>()? {
          match key {
            Field::Len => {
              if len.is_some() { return Err(de::Error::duplicate_field("len")); }
              len = Some(map.next_value()?);
            }
            Field::Bytes => {
              if bytes.is_some() { return Err(de::Error::duplicate_field("bytes")); }
              let helper: DeserializeBytesHelper = map.next_value()?;
              bytes = Some(helper.0);
            }
            Field::Ignore => {
              let _ = map.next_value::<de::IgnoredAny>()?;
            }
          }
        }

        let len = len.ok_or_else(|| de::Error::missing_field("len"))?;
        let bytes = bytes.ok_or_else(|| de::Error::missing_field("bytes"))?;

        reconstruct_arena(len, bytes)
      }
    }

    const FIELDS: &[&str] = &["len", "bytes"];
    deserializer.deserialize_struct("Arena", FIELDS, ArenaVisitor(std::marker::PhantomData))
  }
}

fn reconstruct_arena<T: Copy, E: de::Error>(len: usize, bytes: Vec<u8>) -> Result<Arena<T>, E> {
  let expected_bytes = len * size_of::<T>();
  
  if bytes.len() != expected_bytes {
    return Err(de::Error::invalid_length(
      bytes.len(),
      &format!("{} expected bytes", expected_bytes).as_str()
    ));
  }

  let per_len = Arena::<T>::per_len();
  let mut data = Vec::new();

  if len == 0 {
    data.push(Vec::with_capacity(per_len));
  } else {
    let mut offset = 0;
    while offset < len {
      let chunk_len = (len - offset).min(per_len);
      let mut chunk = Vec::with_capacity(per_len);
      unsafe {
        std::ptr::copy_nonoverlapping(
          bytes.as_ptr().add(offset * size_of::<T>()),
          chunk.as_mut_ptr() as *mut u8,
          chunk_len * size_of::<T>(),
        );
        chunk.set_len(chunk_len);
      }
      data.push(chunk);
      offset += chunk_len;
    }
  }

  Ok(Arena {
    len,
    data,
  })
}


impl<T: Copy> Clone for Arena<T> {
  fn clone(&self) -> Self {
    Self {
      data: self.data.clone(),
      len: self.len,
    }
  }
}

impl<T: Copy> Arena<T> {

  pub fn new() -> Self {
    const {
      assert!(!mem::needs_drop::<T>());
      assert!(size_of::<T>() > 0);
      assert!(size_of::<T>() <= PAGE_SIZE);
    }

		Self { data: vec![ Vec::with_capacity(Self::per_len()) ], len: 0 }
	}


  pub fn push(&mut self, obj: T) -> usize {
    let last_page = self.data.last_mut().expect("En az bir sayfa bulunmalı");

    if last_page.len() == Self::per_len() {
      let mut new_page = Vec::with_capacity(Self::per_len());
      new_page.push(obj);
      self.data.push(new_page);
    } else {
      last_page.push(obj);
    }

    let idx = self.len;
    self.len += 1;
    idx
  }


  pub fn get(&self, idx: usize) -> Option<&T> {
    if idx >= self.len { return None; }
    Some(&self.data[idx / Self::per_len()][idx % Self::per_len()])
  }

  pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
    if idx >= self.len { return None; }
    Some(&mut self.data[idx / Self::per_len()][idx % Self::per_len()])
  }


  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.data.iter().flatten()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
    self.data.iter_mut().flatten()
  }


  pub fn extend_from_slice(&mut self, slice: &[T]) -> Range<usize> {
    let start = self.len;
    if slice.is_empty() { return start..start; }

    let per_len = Self::per_len();
    let mut remaining = slice;

    while !remaining.is_empty() {
      let last_page = self.data.last_mut().expect("En az bir sayfa bulunmalı");
      let available = per_len - last_page.len();

      if available == 0 {
        self.data.push(Vec::with_capacity(per_len));
        continue;
      }

      let take = remaining.len().min(available);
      let (chunk, rest) = remaining.split_at(take);

      self.data.last_mut().unwrap().extend_from_slice(chunk);
      remaining = rest;
    }

    self.len += slice.len();
    start..(start + slice.len())
  }

  pub fn extend_fill(&mut self, value: T, mut count: usize) -> Range<usize> {
    let start = self.len;
    let total = count;
    if count == 0 { return start..start }

    let per_len = Self::per_len();

    while count > 0 {
      let last_page = self.data.last_mut().expect("En az bir sayfa bulunmalı");
      let available = per_len - last_page.len();

      if available == 0 {
        self.data.push(Vec::with_capacity(per_len));
        continue;
      }

      let take = count.min(available);
      last_page.resize(last_page.len() + take, value);
      count -= take;
    }

    self.len += total;
    start..(start + total)
  }


  pub fn range(&self, range: std::ops::Range<usize>) -> impl Iterator<Item = &T> {
    let start = range.start.min(self.len);
    let end = range.end.min(self.len);
    let per_len = Self::per_len();

    let (page_range, start_page, end_page) = if start >= end {
      (0..0, 0, 0)
    } else {
      let sp = start / per_len;
      let ep = (end - 1) / per_len;
      (sp..(ep + 1), sp, ep)
    };

    page_range.flat_map(move |page_idx| {
      let page = &self.data[page_idx];

      let page_start = if page_idx == start_page { start % per_len } else { 0 };

      let page_end = if page_idx == end_page {
        let rem = end % per_len;
        if rem == 0 { page.len() } else { rem }
      } else {
        page.len()
      };

      &page[page_start..page_end]
    })
  }
  

  pub fn len(&self) -> usize { self.len }

  pub fn allocated_len(&self) -> usize { self.data.len() * Self::per_len() }

  pub fn is_empty(&self) -> bool { self.len == 0 }

  const fn per_len() -> usize { PAGE_SIZE / size_of::<T>() }
}


impl<T: Copy> std::ops::Index<usize> for Arena<T> {
  type Output = T;

  fn index(&self, idx: usize) -> &Self::Output {
    self.get(idx).unwrap_or_else(|| panic!("Arena index out of bounds: len is {}, but idx is {}", self.len, idx))
  }
}

impl<T: Copy> std::ops::IndexMut<usize> for Arena<T> {
  fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
    let len = self.len;
    self.get_mut(idx).unwrap_or_else(|| panic!("Arena index out of bounds: len is {}, but idx is {}", len, idx))
  }
}

impl<'a, T: Copy> IntoIterator for &'a Arena<T> {
  type Item = &'a T;
  type IntoIter = std::iter::Flatten<std::slice::Iter<'a, Vec<T>>>;

  fn into_iter(self) -> Self::IntoIter {
    self.data.iter().flatten()
  }
}

impl<T: Copy> Default for Arena<T> {
  fn default() -> Self { Self::new() }
}
