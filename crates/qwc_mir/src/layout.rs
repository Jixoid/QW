use std::num::{NonZeroU32, NonZeroU64};


pub struct LayoutInfo {
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
pub enum LayoutKind {
  SST{align: NonZeroU32, size: NonZeroU64},
  ZST,
  DST{align: NonZeroU32},
  DSAT,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Layout {
  kind: LayoutKind,
  by: LayoutBy,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LayoutBy { SYS, QW, C }



impl Layout {

  // new_*
  pub const fn new_sst(size: u64, align: u32, by: LayoutBy) -> Self {
    assert!(align.is_power_of_two());
    
    let size = NonZeroU64::new(size).unwrap();
    let align = NonZeroU32::new(align).unwrap();

    Layout {
      kind: LayoutKind::SST{ align, size },
      by
    }
  }

  pub const fn new_zst(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::ZST,
      by
    }
  }

  pub const fn new_dst(align: u32, by: LayoutBy) -> Self {
    assert!(align.is_power_of_two());
    
    let align = NonZeroU32::new(align).unwrap();

    Layout {
      kind: LayoutKind::DST{ align },
      by
    }
  }

  pub const fn new_dsat(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::DSAT,
      by
    }
  }


  // is_*
  pub const fn is_sst(&self) -> bool {
    matches!(self.kind, LayoutKind::SST{..})
  }
  
  pub const fn is_zst(&self) -> bool {
    matches!(self.kind, LayoutKind::ZST{..})
  }

  pub const fn is_dst(&self) -> bool {
    matches!(self.kind, LayoutKind::DST{..})
  }

  pub const fn is_dsat(&self) -> bool {
    matches!(self.kind, LayoutKind::DSAT{..})
  }


  // get
  pub const fn size(&self) -> Option<u64> {
    match self.kind {
      LayoutKind::SST{size, ..} => Some(size.get()),
      _ => None
    }
  }
  

  pub const fn align(&self) -> Option<u32> {
    match self.kind {
      LayoutKind::SST{align, ..} => Some(align.get()),
      LayoutKind::DST{align} => Some(align.get()),
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


  // sub
  pub const fn kind(&self) -> LayoutKind {
    self.kind
  }

  pub const fn by(&self) -> LayoutBy {
    self.by
  }

}
