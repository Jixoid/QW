/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::{Message, Status::{self, Success}, Summary, msg::*, ret_err};
use qwc_mir::{Krate, Layout, LayoutInfo, Type, TypeId, TypeKind, TypeRng};


pub struct Layouter;


impl Layouter {
  pub fn layout(sum: &mut Summary, it: &TypeKind, layinfo: &LayoutInfo, cre: &Krate) -> Status<Layout> {
    Self::layout_opt(sum, it, layinfo, cre, false)
  }

  pub fn layout_opt(sum: &mut Summary, it: &TypeKind, layinfo: &LayoutInfo, cre: &Krate, is_c: bool) -> Status<Layout> {
    match *it {
      // Combinated
      TypeKind::Struct(rng) => lay_struct(cre, rng, is_c),

      // Sequential
      TypeKind::Array(ty, count) => lay_array(cre, ty, count),

      // Reference
      TypeKind::Ptr(addrspace) => if let Some(&addr) = layinfo.ptr.get(&addrspace) { Success(addr) } else {
        ret_err!(sum, Message::error_nopos(THE_ADDRESS_SPACE_X_WAS_NOT_FOUND_IN_THIS_TARGET.args(&[&addrspace.to_string()])))
      },

      // Callable
      TypeKind::Fun{..} => if let Some(&addr) = layinfo.ptr.get(&layinfo.fun) { Success(addr) } else {
        panic!()
      },

      _ => todo!("{it:#?}")
    }
  }

  pub fn layout_struct(_sum: &mut Summary, rng: TypeRng, _layinfo: &LayoutInfo, cre: &Krate, is_c: bool) -> Status<Layout> {
    lay_struct(cre, rng, is_c)
  }
}


fn lay_struct(cre: &Krate, rng: TypeRng, is_c: bool) -> Status<Layout> {
  let var = {
    let mut var = vec![];
    
    for id in cre.extra_get(rng) {
      let it: &Type = cre.get(id);
      let lay = it.layout;

      match lay {
        Layout::SST {..} => var.push((it, lay)),
        Layout::ZST {..} => {
          if is_c {
            var.push((it, lay));
          }
        }

        Layout::DST {..} | Layout::DSAT => panic!()
      }
    }
    
    if !is_c {
      var.sort_by(|(_, a), (_, b)| b.align().cmp(&a.align()));
    }
    var
  };

  let (align, size) = {
    let mut align: u32 = 1;
    let mut off: u64 = 0;
    
    for (_, lay) in var {
      if let Some(f_align) = lay.align() {
        align = align.max(f_align);
        off = lay.align_to(off).unwrap_or(off) + lay.aligned_size().unwrap_or(0);
      }
    }

    if is_c && align > 1 {
      off = ((off + align as u64 -1) / align as u64) * align as u64;
    }

    (align, off)
  };

  let it = if size == 0 {
    Layout::new_zst()
  } else {
    Layout::new_sst(size, align)
  };

  Success(it)
}

fn lay_array(cre: &Krate, ty: TypeId, count: u64) -> Status<Layout> {
  let lay = (cre.get(ty) as &Type).layout;

  let it = if lay.is_zst() || count == 0 {
    Layout::new_zst()
  } else {
    Layout::new_sst(lay.aligned_size().unwrap() * (count as u64), lay.align().unwrap())
  };

  Success(it)
}
