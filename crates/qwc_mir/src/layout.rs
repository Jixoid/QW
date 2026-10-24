/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::{NonZeroU32, NonZeroU64};


pub enum ArchBit { B8, B16, B32, B64 }

pub struct LayoutInfo {
  pub arch_bit: ArchBit,
  
  pub ptr_size: Layout,
  
  pub i8_lay: Layout,
  pub i16_lay: Layout,
  pub i32_lay: Layout,
  pub i64_lay: Layout,
  pub i128_lay: Layout,

  pub bf16_lay: Layout,
  pub f16_lay: Layout,
  pub f32_lay: Layout,
  pub f64_lay: Layout,
  pub f128_lay: Layout,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Layout {
  SST{align: NonZeroU32, size: NonZeroU64},
  ZST,
  DST{align: NonZeroU32},
  DSAT,
}



impl Layout {

  // new_*
  pub const fn new_sst(size: u64, align: u32) -> Self {
    assert!(align.is_power_of_two());
    
    let size = NonZeroU64::new(size).unwrap();
    let align = NonZeroU32::new(align).unwrap();

    Layout::SST{ align, size }
  }

  pub const fn new_zst() -> Self {
    Layout::ZST
  }

  pub const fn new_dst(align: u32) -> Self {
    assert!(align.is_power_of_two());
    
    let align = NonZeroU32::new(align).unwrap();

    Layout::DST{ align }
  }

  pub const fn new_dsat() -> Self {
    Layout::DSAT
  }


  // is_*
  pub const fn is_sst(&self) -> bool {
    matches!(self, Layout::SST{..})
  }
  
  pub const fn is_zst(&self) -> bool {
    matches!(self, Layout::ZST{..})
  }

  pub const fn is_dst(&self) -> bool {
    matches!(self, Layout::DST{..})
  }

  pub const fn is_dsat(&self) -> bool {
    matches!(self, Layout::DSAT{..})
  }


  // get
  pub const fn size(&self) -> Option<u64> {
    match self {
      Layout::SST{size, ..} => Some(size.get()),
      _ => None
    }
  }
  

  pub const fn align(&self) -> Option<u32> {
    match self {
      Layout::SST{align, ..} => Some(align.get()),
      Layout::DST{align} => Some(align.get()),
      _ => None
    }
  }


  pub const fn aligned_size(&self) -> Option<u64> {
    if let Some(size) = self.size() {
      if let Some(align) = self.align() {
        let align = align as u64;

        return Some((size +align -1) & !(align -1))
      }
    }

    None
  }


  pub const fn align_to(&self, offset: u64) -> Option<u64> {
    if let Some(align) = self.align() {
      let align = align as u64;
      debug_assert!(align.is_power_of_two());

      return Some((offset +align -1) & !(align -1))
    }
    
    None
  }

}
