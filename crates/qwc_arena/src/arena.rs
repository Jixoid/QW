use core::slice;
use std::{mem, ops::Range};

use serde::ser::{Serialize, Serializer, SerializeStruct};

const PAGE_SIZE: usize = 1 * 1024 * 1024; // 1 MB


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
