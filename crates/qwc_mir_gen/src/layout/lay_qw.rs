use qwc_mir::{Krate, Layout, LayoutBy, LayoutInfo, Layouter, Type, TypeId, TypeKind};


pub struct LayouterQW;

impl Layouter for LayouterQW {
  fn layout(it: &TypeKind, layinfo: &LayoutInfo, krate: &Krate) -> Layout {
    match *it {
      // Combinated
      TypeKind::Struct(rng) => {
        let var = {
          let mut var = vec![];
          
          for id in krate.extra_get(rng) {
            let it: &Type = krate.get(TypeId::new_from(id));
            let lay = it.layout;
            
            if !lay.is_zst() { var.push((it, lay)) }
          }
          
          var.sort_by(|(_, a), (_, b)| b.align().cmp(&a.align()));
          var
        };

        let (align, size) = {
          let mut align: usize = 1;
          let mut off: usize = 0;
          
          for (_, lay) in var {
            align = align.max(lay.align().get());
            off = lay.align_to(off) + lay.aligned_size();
          }

          (align, off)
        };

        Layout::new(size, align, Self::lay_by())
      }


      // Sequential
      TypeKind::Array(ty, count) => {
        let lay = (krate.get(ty) as &Type).layout;

        Layout::new(lay.aligned_size() * (count as usize), lay.align().get(), Self::lay_by())
      }


      // Reference
      TypeKind::Ptr => layinfo.ptr_size,


      // Callable
      TypeKind::Fun{..} => layinfo.ptr_size,
      

      _ => todo!("{it:#?}")
    }
  }

  fn lay_by() -> LayoutBy { LayoutBy::QW }
}
