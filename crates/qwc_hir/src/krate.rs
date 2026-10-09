/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{num::NonZero, slice};
use serde::Serialize;

use qwc_arena::Arena;

use crate::{AnyId, AnyRng, Expr, Item, ItemId, PrimTypes, Rng, Thing, Type, id::{HirId, HirKind, NodeKind, SpecAny}};



#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CID (pub(crate) NonZero<u16>);


#[derive(Clone)]
pub struct Deps {
  imod: Vec<Krate>,
  prims: Option<PrimTypes>,
}

impl Deps {
  pub fn new() -> Self {
    Self { imod: vec![], prims: None }
  }

  pub fn set_prims(&mut self, prims: PrimTypes) {
    self.prims = Some(prims);
  }

  pub fn prims(&self) -> Option<&PrimTypes> {
    self.prims.as_ref()
  }

  pub fn add(&mut self, cre: Krate) -> CID {
    self.imod.push(cre);

    CID( NonZero::<u16>::new(u16::try_from(self.imod.len()).unwrap()).unwrap() )
  }

  pub fn get_next_id(&mut self) -> CID {
    CID( NonZero::<u16>::new(u16::try_from(self.imod.len() +1).unwrap()).unwrap() )
  }

  pub fn get(&self, cid: CID) -> &Krate {
    &self.imod[cid.0.get() as usize -1]
  }
}




#[derive(Serialize, Clone)]
pub struct Krate {
  // Root
  root: Option<ItemId>,

  // cid
  cid: CID,

  // Arena
  list_type: Arena<Type>,
  list_expr: Arena<Expr>,
  list_item: Arena<Item>,
  list_thin: Arena<Thing>,

  extra_data: (Arena<HirId<SpecAny>>, Arena<NodeKind>),
}

impl Krate {

  // New
  pub fn new(cid: CID) -> Self {
    Self{
      root: None,

      cid,

      list_type: Arena::new(),
      list_expr: Arena::new(),
      list_item: Arena::new(),
      list_thin: Arena::new(),
      
      extra_data: (Arena::new(), Arena::new()),
    }
  }

  pub fn cid(&self) -> CID {
    self.cid
  }


  // Root
  pub fn root(&self) -> Option<ItemId> {
    self.root
  }

  pub fn set_root(&mut self, id: ItemId) {
    self.root = Some(id)
  }


  // Arena
  pub fn push<T: HirKind, P: PushApi<T>>(&mut self, obj: P) -> HirId<T> { P::push(self, obj) }
  pub fn get<T: HirKind + GetApi<T>>(&self, id: HirId<T>) -> &T { T::get(self, id) }
  pub fn get_mut<T: HirKind + GetApi<T>>(&mut self, id: HirId<T>) -> &mut T { T::get_mut(self, id) }


  // Extra
  pub fn extra<T: HirKind>(&mut self, vec: &[HirId<T>]) -> Rng<T> {
    let (ids, kds) = &mut self.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const HirId<SpecAny>, vec.len()) };

    let irng = ids.extend_from_slice(vec);
    let krng = kds.extend_fill(T::kind(), vec.len());
    debug_assert_eq!(irng, krng);

    Rng::new(self.cid, u32::try_from(irng.start).unwrap(), u32::try_from(irng.end).unwrap())
  }

  pub fn extra_any(&mut self, vec: &[AnyId]) -> AnyRng {
    let (ids, kds) = &mut self.extra_data;
    let start = ids.len();
    for any in vec {
      ids.push(any.id());
      kds.push(any.kind());
    }
    AnyRng::new(self.cid, u32::try_from(start).unwrap(), u32::try_from(ids.len()).unwrap())
  }

  pub fn extra_get<T: HirKind>(&self, rng: Rng<T>) -> impl Iterator<Item = HirId<T>> {
    assert_eq!(rng.cid(), self.cid);

    let (ids, kinds) = &self.extra_data;
    let range = rng.range();

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| HirId::<T>::new_from((id, kind)))
  }
  
  pub fn extra_any_get(&self, rng: AnyRng) -> impl Iterator<Item = AnyId> {
    assert_eq!(rng.cid(), self.cid);
    
    let (ids, kinds) = &self.extra_data;
    let range = rng.range();

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| AnyId::new_from((id, kind)))
  }


  // Size
  pub fn size_used<T: SizeApi>(&self) -> usize { T::size_used(&self) }
  pub fn size_alloc<T: SizeApi>(&self) -> usize { T::size_alloc(&self) }
  
  pub fn size_all_used(&self) -> usize {
    Self::size_used::<Type>(&self) + Self::size_used::<Expr>(&self) + Self::size_used::<Item>(&self) + Self::size_used::<Thing>(&self) + Self::size_used::<AnyId>(&self)
  }

  pub fn size_all_alloc(&self) -> usize {
    Self::size_alloc::<Type>(&self) + Self::size_alloc::<Expr>(&self) + Self::size_alloc::<Item>(&self) + Self::size_alloc::<Thing>(&self) + Self::size_alloc::<AnyId>(&self)
  }

}



// push & get
pub trait PushApi<T: HirKind> {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T>;
}

pub trait GetApi<T: HirKind> {
  fn get<'a>(krate: &'a Krate, id: HirId<T>) -> &'a Self;
  fn get_mut<'a>(krate: &'a mut Krate, id: HirId<T>) -> &'a mut Self;
}


impl<T: HirKind> PushApi<T> for Type {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> { HirId::<T>::new(krate.cid, u32::try_from(krate.list_type.push(obj)).unwrap()) }
}

impl<T: HirKind> GetApi<T> for Type {
  fn get<'a>(krate: &'a Krate, id: HirId<T>) -> &'a Self { assert_eq!(id.cid(), krate.cid); &krate.list_type[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: HirId<T>) -> &'a mut Self { assert_eq!(id.cid(), krate.cid); &mut krate.list_type[id.idx() as usize] }
}


impl<T: HirKind> PushApi<T> for Expr {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> { HirId::<T>::new(krate.cid, u32::try_from(krate.list_expr.push(obj)).unwrap()) }
}

impl<T: HirKind> GetApi<T> for Expr {
  fn get<'a>(krate: &'a Krate, id: HirId<T>) -> &'a Self { assert_eq!(id.cid(), krate.cid); &krate.list_expr[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: HirId<T>) -> &'a mut Self { assert_eq!(id.cid(), krate.cid); &mut krate.list_expr[id.idx() as usize] }
}


impl<T: HirKind> PushApi<T> for Item {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> { HirId::<T>::new(krate.cid, u32::try_from(krate.list_item.push(obj)).unwrap()) }
}

impl<T: HirKind> GetApi<T> for Item {
  fn get<'a>(krate: &'a Krate, id: HirId<T>) -> &'a Self { assert_eq!(id.cid(), krate.cid); &krate.list_item[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: HirId<T>) -> &'a mut Self { assert_eq!(id.cid(), krate.cid); &mut krate.list_item[id.idx() as usize] }
}


impl<T: HirKind> PushApi<T> for Thing {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> { HirId::<T>::new(krate.cid, u32::try_from(krate.list_thin.push(obj)).unwrap()) }
}

impl<T: HirKind> GetApi<T> for Thing {
  fn get<'a>(krate: &'a Krate, id: HirId<T>) -> &'a Self { assert_eq!(id.cid(), krate.cid); &krate.list_thin[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: HirId<T>) -> &'a mut Self { assert_eq!(id.cid(), krate.cid); &mut krate.list_thin[id.idx() as usize] }
}


// size api
pub trait SizeApi {
  fn size_used(cre: &Krate) -> usize;
  fn size_alloc(cre: &Krate) -> usize;
}

impl SizeApi for Type {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_type.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_type.allocated_len() }
}

impl SizeApi for Expr {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_expr.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_expr.allocated_len() }
}

impl SizeApi for Item {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_item.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_item.allocated_len() }
}

impl SizeApi for Thing {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_thin.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_thin.allocated_len() }
}

impl SizeApi for AnyId {
  fn size_used(cre: &Krate) -> usize { (size_of::<HirId<SpecAny>>() * cre.extra_data.0.len()) + (size_of::<NodeKind>() * cre.extra_data.1.len()) }
  fn size_alloc(cre: &Krate) -> usize { (size_of::<HirId<SpecAny>>() * cre.extra_data.0.allocated_len()) + (size_of::<NodeKind>() * cre.extra_data.1.allocated_len()) }
}
