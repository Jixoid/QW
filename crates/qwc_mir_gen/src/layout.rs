/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir as hir;
use qwc_mir::{Krate, Layout, LayoutBy, LayoutInfo, LayoutKind, Type, TypeId, TypeKind, TypeRng};


pub struct Layouter;

pub enum LayRepr { QW, C }


impl Layouter {
  pub fn layout(it: &TypeKind, layinfo: &LayoutInfo, cre: &Krate, repr: hir::LayoutBy) -> Layout {
    let repr = match repr { hir::LayoutBy::SYS | hir::LayoutBy::QW => LayRepr::QW, hir::LayoutBy::C => LayRepr::C };
    
    match *it {
      // Combinated
      TypeKind::Struct(rng) => match repr {
        LayRepr::QW => lay_qw_struct(cre, rng),
        LayRepr::C  => lay_c_struct(cre, rng),
      }

      // Sequential
      TypeKind::Array(ty, count) => match repr {
        LayRepr::QW => lay_qw_array(cre, ty, count),
        LayRepr::C  => lay_c_array(cre, ty, count),
      }

      // Reference
      TypeKind::Ptr => layinfo.ptr_size,

      // Callable
      TypeKind::Fun{..} => layinfo.ptr_size,

      _ => todo!("{it:#?}")
    }
  }
}


fn lay_qw_struct(cre: &Krate, rng: TypeRng) -> Layout {
  let var = {
    let mut var = vec![];
    
    for id in cre.extra_get(rng) {
      let it: &Type = cre.get(id);
      let lay = it.layout;

      match lay.kind() {
        LayoutKind::SST {..} => var.push((it, lay)),
        LayoutKind::ZST {..} => {/* ignore */}

        LayoutKind::DST {..} | LayoutKind::DSAT => panic!()
      }
    }
    
    var.sort_by(|(_, a), (_, b)| b.align().cmp(&a.align()));
    var
  };

  let (align, size) = {
    let mut align: u32 = 1;
    let mut off: u64 = 0;
    
    for (_, lay) in var {
      align = align.max(lay.align().unwrap());
      off = lay.align_to(off).unwrap() + lay.aligned_size().unwrap();
    }

    (align, off)
  };

  if size == 0 {
    Layout::new_zst(LayoutBy::QW)
  } else {
    Layout::new_sst(size, align, LayoutBy::QW)
  }
}

fn lay_c_struct(cre: &Krate, rng: TypeRng) -> Layout {
  let (align, size) = {
    let mut align: u32 = 1;
    let mut off: u64 = 0;
    
    for id in cre.extra_get(rng) {
      let it: &Type = cre.get(id);
      let lay = it.layout;

      match lay.kind() {
        LayoutKind::SST {..} => (),
        LayoutKind::ZST {..} | LayoutKind::DST {..} | LayoutKind::DSAT => panic!()
      }
      
      align = align.max(lay.align().unwrap());
      off = lay.align_to(off).unwrap() + lay.aligned_size().unwrap();
    }
    
    (align, off)
  };

  Layout::new_sst(size.max(1), align, LayoutBy::C)
}


fn lay_qw_array(cre: &Krate, ty: TypeId, count: u32) -> Layout {
  let lay = (cre.get(ty) as &Type).layout;

  if lay.is_zst() || count == 0 {
    Layout::new_zst(LayoutBy::QW)
  } else {
    Layout::new_sst(lay.aligned_size().unwrap() * (count as u64), lay.align().unwrap(), LayoutBy::QW)
  }
}

fn lay_c_array(cre: &Krate, ty: TypeId, count: u32) -> Layout {
  let lay = (cre.get(ty) as &Type).layout;

  Layout::new_sst(lay.aligned_size().unwrap() * (count as u64), lay.align().unwrap(), LayoutBy::C)
}
