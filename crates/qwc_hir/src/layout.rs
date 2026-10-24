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
  
  pub str_lay: Layout,

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


#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LayoutKind { Static, Meta, DST, DSAT }



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Layout {
  kind: LayoutKind,
  inhabited: bool,
}

impl Layout {
  pub const STATIC: Layout = Layout{kind: LayoutKind::Static, inhabited: false};
  pub const META: Layout = Layout{kind: LayoutKind::Meta, inhabited: false};
  pub const DST: Layout = Layout{kind: LayoutKind::DST, inhabited: false};
  pub const DSAT: Layout = Layout{kind: LayoutKind::DSAT, inhabited: false};
}


impl Layout {

  // new_*
  pub const fn new_inhabited() -> Self {
    Layout {
      kind: LayoutKind::Static,
      inhabited: true,
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
  pub(crate) const fn kind(&self) -> LayoutKind {
    self.kind
  }

  #[cfg(feature = "unit-internal")]
  pub const fn from_raw(kind: u8, inhabited: bool) -> Self {
    let kind = match kind {
      0 => LayoutKind::Static,
      1 => LayoutKind::Meta,
      2 => LayoutKind::DST,
      _ => LayoutKind::DSAT,
    };
    Self { kind, inhabited }
  }

  #[cfg(feature = "unit-internal")]
  pub const fn raw_kind(&self) -> u8 {
    match self.kind {
      LayoutKind::Static => 0,
      LayoutKind::Meta => 1,
      LayoutKind::DST => 2,
      LayoutKind::DSAT => 3,
    }
  }

}
