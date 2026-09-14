use qwc_mir::{Krate, Layout, LayoutBy, LayoutInfo, Layouter, Type, TypeId, TypeKind};


pub struct LayouterC;

impl Layouter for LayouterC {
  fn layout(it: &TypeKind, layinfo: &LayoutInfo, krate: &Krate) -> Layout {
    let lay = match *it {
      // Combinated
      TypeKind::Struct(rng) => {
        let (align, size) = {
          let mut align: usize = 1;
          let mut off: usize = 0;
          
          for id in krate.extra_get(rng) {
            let it: &Type = krate.get(TypeId::new_from(id));
            let lay = Self::layout(&it.kind, layinfo, krate);

            if lay.is_zst() { panic!() }
            
            align = align.max(lay.align_byte().get());
            off = lay.align_to_byte(off) + lay.aligned_size_byte();
          }
          
          (align, off)
        };

        Layout::new(size*8, align*8, Self::lay_by())
      }


      // Sequential
      TypeKind::Array(ty, count) => {
        let lay = Self::layout(&(krate.get(ty) as &Type).kind, layinfo, krate);

        Layout::new(lay.aligned_size() * (count as usize), lay.align().get(), Self::lay_by())
      }


      // Reference
      TypeKind::Ptr(..) => layinfo.ptr_size,

      _ => todo!("{it:#?}")
    };

    assert_eq!(lay.size() % 8, 0);
    assert_eq!(lay.align().get() % 8, 0);

    lay
  }

  fn lay_by() -> LayoutBy { LayoutBy::C }
}
