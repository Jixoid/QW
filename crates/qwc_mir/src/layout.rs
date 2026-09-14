use std::num::NonZeroUsize;

use crate::{Krate, TypeKind};


pub trait Layouter {
  fn layout(it: &TypeKind, layinfo: &LayoutInfo, krate: &Krate) -> Layout;

  fn lay_by() -> LayoutBy;
}

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
pub struct Layout {
  size: usize,
  align: NonZeroUsize,
  by: LayoutBy,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LayoutBy { SYS, QW, C }



impl Layout {

  pub const fn new(size: usize, align: usize, by: LayoutBy) -> Self {
    let align = NonZeroUsize::new(align).unwrap();
    assert!(align.get().is_power_of_two());
    Self { size, align, by }
  }


  pub const fn is_zst(&self) -> bool {
    self.size == 0
  }


  pub const fn size(&self) -> usize {
    self.size
  }
  
  pub const fn size_byte(&self) -> usize {
    self.size.div_ceil(8)
  }


  pub const fn align(&self) -> NonZeroUsize {
    self.align
  }

  pub const fn align_byte(&self) -> NonZeroUsize {
    self.align.div_ceil(unsafe {NonZeroUsize::new_unchecked(8)})
  }


  pub const fn aligned_size(&self) -> usize {
    let a = self.align.get();
    (self.size + a -1) & !(a -1)
  }

  pub const fn aligned_size_byte(&self) -> usize {
    self.aligned_size().div_ceil(8)
  }


  pub const fn align_to(&self, offset: usize) -> usize {
    let a = self.align.get();
    debug_assert!(a.is_power_of_two());
    (offset + a -1) & !(a -1)
  }

  pub const fn align_to_byte(&self, offset: usize) -> usize {
    let a = self.align_byte().get();
    debug_assert!(a.is_power_of_two());
    (offset + a -1) & !(a -1)
  }

}
