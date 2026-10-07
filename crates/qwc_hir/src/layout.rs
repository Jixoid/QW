/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


pub struct LayoutInfo {
  pub ptr_size: Layout,
  
  pub bool_lay: Layout,

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
pub enum LayoutKind { Static, Meta, DST, DSAT }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum LayoutBy { SYS, QW, C }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Layout {
  kind: LayoutKind,
  inhabited: bool,
  by: LayoutBy,
}


impl Layout {

  // new_*
  pub const fn new_inhabited(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::Static,
      inhabited: true,
      by
    }
  }

  pub const fn new_static(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::Static,
      inhabited: false,
      by
    }
  }

  pub const fn new_meta(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::Meta,
      inhabited: false,
      by
    }
  }

  pub const fn new_dst(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::DST,
      inhabited: false,
      by
    }
  }

  pub const fn new_dsat(by: LayoutBy) -> Self {
    Layout {
      kind: LayoutKind::DSAT,
      inhabited: false,
      by
    }
  }


  // is_*
  pub const fn is_inhabited(&self) -> bool {
    self.inhabited
  }

  pub const fn is_static(&self) -> bool {
    matches!(self.kind, LayoutKind::Static)
  }

  pub const fn is_meta(&self) -> bool {
    matches!(self.kind, LayoutKind::Meta)
  }

  pub const fn is_dynamic(&self) -> bool {
    matches!(self.kind, LayoutKind::DST | LayoutKind::DSAT)
  }
  

  // sub
  pub const fn kind(&self) -> LayoutKind {
    self.kind
  }

  pub const fn by(&self) -> LayoutBy {
    self.by
  }

}
