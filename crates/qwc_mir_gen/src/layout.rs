use qwc_mir::{Krate, Layout, LayoutBy, LayoutInfo, LayoutKind, Rng, Type, TypeId, TypeKind};


pub struct Layouter;

pub enum LayRepr { QW, C }


impl Layouter {
  pub fn layout(it: &TypeKind, layinfo: &LayoutInfo, cre: &Krate, repr: Option<LayRepr>) -> Layout {
    let repr = repr.unwrap_or(LayRepr::QW);
    let layby = match repr { LayRepr::QW => LayoutBy::QW, LayRepr::C => LayoutBy::C };
    
    match *it {
      // Combinated
      TypeKind::Struct(rng) => match repr {
        LayRepr::QW => lay_qw_struct(cre, layby, rng),
        LayRepr::C  => lay_c_struct(cre, layby, rng),
      }


      // Sequential
      TypeKind::Array(ty, count) => match repr {
        LayRepr::QW => lay_qw_array(cre, layby, ty, count),
        LayRepr::C  => lay_c_array(cre, layby, ty, count),
      }


      // Reference
      TypeKind::Ptr => layinfo.ptr_size,


      // Callable
      TypeKind::Fun{..} => layinfo.ptr_size,
      

      _ => todo!("{it:#?}")
    }
  }
}


fn lay_qw_struct(cre: &Krate, layby: LayoutBy, rng: Rng) -> Layout {
  let var = {
    let mut var = vec![];
    
    for id in cre.extra_get(rng) {
      let it: &Type = cre.get(TypeId::new_from(id));
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
    Layout::new_zst(layby)
  } else {
    Layout::new_sst(size, align, layby)
  }
}

fn lay_c_struct(cre: &Krate, layby: LayoutBy, rng: Rng) -> Layout {
  let (align, size) = {
    let mut align: u32 = 1;
    let mut off: u64 = 0;
    
    for id in cre.extra_get(rng) {
      let it: &Type = cre.get(TypeId::new_from(id));
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

  Layout::new_sst(size, align, layby)
}


fn lay_qw_array(cre: &Krate, layby: LayoutBy, ty: TypeId, count: u32) -> Layout {
  let lay = (cre.get(ty) as &Type).layout;

  if lay.is_zst() || count == 0 {
    Layout::new_zst(layby)
  } else {
    Layout::new_sst(lay.aligned_size().unwrap() * (count as u64), lay.align().unwrap(), layby)
  }
}

fn lay_c_array(cre: &Krate, layby: LayoutBy, ty: TypeId, count: u32) -> Layout {
  let lay = (cre.get(ty) as &Type).layout;

  Layout::new_sst(lay.aligned_size().unwrap() * (count as u64), lay.align().unwrap(), layby)
}
