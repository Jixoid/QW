/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{num::NonZero, slice};
use serde::{Deserialize, Serialize};

use qwc_arena::Arena;

use crate::{AnyId, AnyRng, DefPath, DefPathId, Expr, Item, ItemId, PrimOps, PrimTypes, Rng, Thing, Type, id::{HirId, HirKind, NodeKind, SpecAny}};



#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CID (pub(crate) NonZero<u16>);

impl CID {
  #[cfg(feature = "unit-internal")]
  pub fn from_raw(raw: u16) -> Self {
    Self(NonZero::<u16>::new(raw).unwrap())
  }

  pub fn raw(&self) -> u16 {
    self.0.get()
  }
}


#[derive(Default, Clone)]
pub struct Deps {
  imod: Vec<Krate>,
  prims: Option<PrimTypes>,
  prim_op: Option<PrimOps>,
}

impl Deps {
  pub fn set_prims(&mut self, prims: PrimTypes) {
    self.prims = Some(prims);
  }

  pub fn prims(&self) -> Option<&PrimTypes> {
    self.prims.as_ref()
  }

  pub fn set_prim_op(&mut self, prim_op: PrimOps) {
    self.prim_op = Some(prim_op);
  }

  pub fn prim_op(&self) -> Option<&PrimOps> {
    self.prim_op.as_ref()
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



pub trait LazyLoader: Send + Sync {
  fn load_type(&self, idx: usize) -> Type;
  fn load_expr(&self, idx: usize) -> Expr;
  fn load_item(&self, idx: usize) -> Item;
  fn load_thing(&self, idx: usize) -> Thing;
  fn load_dpath(&self, idx: usize) -> DefPath;
}

pub struct LazyKrateData {
  pub loader: std::sync::Arc<dyn LazyLoader>,
  pub types: Box<[std::sync::OnceLock<Type>]>,
  pub exprs: Box<[std::sync::OnceLock<Expr>]>,
  pub items: Box<[std::sync::OnceLock<Item>]>,
  pub thins: Box<[std::sync::OnceLock<Thing>]>,
  pub dpaths: Box<[std::sync::OnceLock<DefPath>]>,
  pub extra_ids: Vec<HirId<SpecAny>>,
  pub extra_kinds: Vec<NodeKind>,
  pub impl_items: Vec<ItemId>,
}

#[derive(Clone)]
pub struct EagerData {
  list_type: Arena<Type>,
  list_expr: Arena<Expr>,
  list_item: Arena<Item>,
  list_thin: Arena<Thing>,
  list_dpath: Arena<DefPath>,
  extra_data: (Arena<HirId<SpecAny>>, Arena<NodeKind>),
}

#[derive(Clone)]
pub enum KrateData {
  Eager(EagerData),
  Lazy(std::sync::Arc<LazyKrateData>),
}

#[derive(Clone)]
pub struct Krate {
  // Root
  root: Option<ItemId>,

  // cid
  cid: CID,

  data: KrateData,
}

impl Krate {

  // New
  pub fn new(cid: CID) -> Self {
    Self {
      root: None,
      cid,
      data: KrateData::Eager(EagerData {
        list_type: Arena::new(),
        list_expr: Arena::new(),
        list_item: Arena::new(),
        list_thin: Arena::new(),
        list_dpath: Arena::new(),
        extra_data: (Arena::new(), Arena::new()),
      }),
    }
  }

  #[cfg(feature = "unit-internal")]
  pub fn new_lazy(
    cid: CID,
    root: Option<ItemId>,
    loader: std::sync::Arc<dyn LazyLoader>,
    counts: (usize, usize, usize, usize, usize),
    extra_data: (Vec<HirId<SpecAny>>, Vec<NodeKind>),
    impl_items: Vec<ItemId>,
  ) -> Self {
    let (t_cnt, e_cnt, i_cnt, th_cnt, d_cnt) = counts;
    Self {
      root,
      cid,
      data: KrateData::Lazy(std::sync::Arc::new(LazyKrateData {
        loader,
        types: (0..t_cnt).map(|_| std::sync::OnceLock::new()).collect(),
        exprs: (0..e_cnt).map(|_| std::sync::OnceLock::new()).collect(),
        items: (0..i_cnt).map(|_| std::sync::OnceLock::new()).collect(),
        thins: (0..th_cnt).map(|_| std::sync::OnceLock::new()).collect(),
        dpaths: (0..d_cnt).map(|_| std::sync::OnceLock::new()).collect(),
        extra_ids: extra_data.0,
        extra_kinds: extra_data.1,
        impl_items,
      })),
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

  // Arena push & get
  pub fn push<T: HirKind, P: PushApi<T>>(&mut self, obj: P) -> HirId<T> { P::push(self, obj) }
  pub fn get<T: HirKind + GetApi<T>>(&self, id: HirId<T>) -> &T { T::get(self, id) }
  pub fn get_mut<T: HirKind + GetApi<T>>(&mut self, id: HirId<T>) -> &mut T { T::get_mut(self, id) }

  // Extra
  pub fn extra<T: HirKind>(&mut self, vec: &[HirId<T>]) -> Rng<T> {
    let eager = match &mut self.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot add extra to lazy krate"),
    };
    let (ids, kds) = &mut eager.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const HirId<SpecAny>, vec.len()) };

    let irng = ids.extend_from_slice(vec);
    let krng = kds.extend_fill(T::kind(), vec.len());
    debug_assert_eq!(irng, krng);

    Rng::new(self.cid, u32::try_from(irng.start).unwrap(), u32::try_from(irng.end).unwrap())
  }

  pub fn extra_any(&mut self, vec: &[AnyId]) -> AnyRng {
    let eager = match &mut self.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot add extra to lazy krate"),
    };
    let (ids, kds) = &mut eager.extra_data;
    let start = ids.len();
    for any in vec {
      ids.push(any.id());
      kds.push(any.kind());
    }
    AnyRng::new(self.cid, u32::try_from(start).unwrap(), u32::try_from(ids.len()).unwrap())
  }

  pub fn extra_get<T: HirKind>(&self, rng: Rng<T>) -> Box<dyn Iterator<Item = HirId<T>> + '_> {
    assert_eq!(rng.cid(), self.cid);
    let range = rng.range();
    match &self.data {
      KrateData::Eager(eager) => {
        let (ids, kinds) = &eager.extra_data;
        Box::new(
          std::iter::zip(
            ids.range(range.clone()),
            kinds.range(range),
          )
          .map(|(&id, &kind)| HirId::<T>::new_from((id, kind)))
        )
      }
      KrateData::Lazy(lazy) => {
        Box::new(
          std::iter::zip(
            lazy.extra_ids[range.clone()].iter(),
            lazy.extra_kinds[range].iter(),
          )
          .map(|(&id, &kind)| HirId::<T>::new_from((id, kind)))
        )
      }
    }
  }

  pub fn extra_any_get(&self, rng: AnyRng) -> Box<dyn Iterator<Item = AnyId> + '_> {
    assert_eq!(rng.cid(), self.cid);
    let range = rng.range();
    match &self.data {
      KrateData::Eager(eager) => {
        let (ids, kinds) = &eager.extra_data;
        Box::new(
          std::iter::zip(
            ids.range(range.clone()),
            kinds.range(range),
          )
          .map(|(&id, &kind)| AnyId::new_from((id, kind)))
        )
      }
      KrateData::Lazy(lazy) => {
        Box::new(
          std::iter::zip(
            lazy.extra_ids[range.clone()].iter(),
            lazy.extra_kinds[range].iter(),
          )
          .map(|(&id, &kind)| AnyId::new_from((id, kind)))
        )
      }
    }
  }

  pub fn impl_items(&self) -> Vec<ItemId> {
    match &self.data {
      KrateData::Eager(eager) => {
        let cid = self.cid;
        let mut list = Vec::new();
        for idx in 0..eager.list_item.len() {
          if matches!(eager.list_item[idx].kind, crate::ItemKind::Impl { .. }) {
            list.push(ItemId::new(cid, idx as u32));
          }
        }
        list
      }
      KrateData::Lazy(lazy) => {
        lazy.impl_items.clone()
      }
    }
  }

  // Size
  pub fn size_used<T: SizeApi>(&self) -> usize { T::size_used(self) }
  pub fn size_alloc<T: SizeApi>(&self) -> usize { T::size_alloc(self) }

  pub fn size_all_used(&self) -> usize {
    Self::size_used::<Type>(self) + Self::size_used::<Expr>(self) + Self::size_used::<Item>(self) + Self::size_used::<Thing>(self) + Self::size_used::<DefPath>(self) + Self::size_used::<AnyId>(self)
  }

  pub fn size_all_alloc(&self) -> usize {
    Self::size_alloc::<Type>(self) + Self::size_alloc::<Expr>(self) + Self::size_alloc::<Item>(self) + Self::size_alloc::<Thing>(self) + Self::size_alloc::<DefPath>(self) + Self::size_alloc::<AnyId>(self)
  }

}

#[cfg(feature = "unit-internal")]
impl Krate {
  pub fn types_count(&self) -> usize {
    match &self.data {
      KrateData::Eager(eager) => eager.list_type.len(),
      KrateData::Lazy(lazy) => lazy.types.len(),
    }
  }

  pub fn exprs_count(&self) -> usize {
    match &self.data {
      KrateData::Eager(eager) => eager.list_expr.len(),
      KrateData::Lazy(lazy) => lazy.exprs.len(),
    }
  }

  pub fn items_count(&self) -> usize {
    match &self.data {
      KrateData::Eager(eager) => eager.list_item.len(),
      KrateData::Lazy(lazy) => lazy.items.len(),
    }
  }

  pub fn thins_count(&self) -> usize {
    match &self.data {
      KrateData::Eager(eager) => eager.list_thin.len(),
      KrateData::Lazy(lazy) => lazy.thins.len(),
    }
  }

  pub fn dpaths_count(&self) -> usize {
    match &self.data {
      KrateData::Eager(eager) => eager.list_dpath.len(),
      KrateData::Lazy(lazy) => lazy.dpaths.len(),
    }
  }

  pub fn get_type_by_idx(&self, idx: usize) -> &Type {
    match &self.data {
      KrateData::Eager(eager) => &eager.list_type[idx],
      KrateData::Lazy(lazy) => lazy.types[idx].get_or_init(|| lazy.loader.load_type(idx)),
    }
  }

  pub fn get_expr_by_idx(&self, idx: usize) -> &Expr {
    match &self.data {
      KrateData::Eager(eager) => &eager.list_expr[idx],
      KrateData::Lazy(lazy) => lazy.exprs[idx].get_or_init(|| lazy.loader.load_expr(idx)),
    }
  }

  pub fn get_item_by_idx(&self, idx: usize) -> &Item {
    match &self.data {
      KrateData::Eager(eager) => &eager.list_item[idx],
      KrateData::Lazy(lazy) => lazy.items[idx].get_or_init(|| lazy.loader.load_item(idx)),
    }
  }

  pub fn get_thin_by_idx(&self, idx: usize) -> &Thing {
    match &self.data {
      KrateData::Eager(eager) => &eager.list_thin[idx],
      KrateData::Lazy(lazy) => lazy.thins[idx].get_or_init(|| lazy.loader.load_thing(idx)),
    }
  }

  pub fn get_dpath_by_idx(&self, idx: usize) -> &DefPath {
    match &self.data {
      KrateData::Eager(eager) => &eager.list_dpath[idx],
      KrateData::Lazy(lazy) => lazy.dpaths[idx].get_or_init(|| lazy.loader.load_dpath(idx)),
    }
  }

  pub fn extra_raw(&self) -> (Vec<HirId<SpecAny>>, Vec<NodeKind>) {
    match &self.data {
      KrateData::Eager(eager) => {
        let (ids, kinds) = &eager.extra_data;
        let id_vec = (0..ids.len()).map(|i| ids[i]).collect();
        let kd_vec = (0..kinds.len()).map(|i| kinds[i]).collect();
        (id_vec, kd_vec)
      }
      KrateData::Lazy(lazy) => {
        (lazy.extra_ids.clone(), lazy.extra_kinds.clone())
      }
    }
  }
}



// Path Api
pub trait PathApi {
  fn path(&self) -> DefPathId;
}


// push & get
pub trait PushApi<T: HirKind> {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T>;
}

pub trait GetApi<T: HirKind> {
  fn get(krate: &Krate, id: HirId<T>) -> &Self;
  fn get_mut(krate: &mut Krate, id: HirId<T>) -> &mut Self;
}


impl<T: HirKind> PushApi<T> for Type {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> {
    let eager = match &mut krate.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot push to lazy krate"),
    };
    HirId::<T>::new(krate.cid, u32::try_from(eager.list_type.push(obj)).unwrap())
  }
}

impl<T: HirKind> GetApi<T> for Type {
  fn get(krate: &Krate, id: HirId<T>) -> &Self {
    assert_eq!(id.cid(), krate.cid);
    match &krate.data {
      KrateData::Eager(eager) => &eager.list_type[id.idx() as usize],
      KrateData::Lazy(lazy) => lazy.types[id.idx() as usize].get_or_init(|| lazy.loader.load_type(id.idx() as usize)),
    }
  }
  fn get_mut(krate: &mut Krate, id: HirId<T>) -> &mut Self {
    assert_eq!(id.cid(), krate.cid);
    match &mut krate.data {
      KrateData::Eager(eager) => &mut eager.list_type[id.idx() as usize],
      KrateData::Lazy(_) => panic!("cannot get_mut on lazy krate"),
    }
  }
}


impl<T: HirKind> PushApi<T> for Expr {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> {
    let eager = match &mut krate.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot push to lazy krate"),
    };
    HirId::<T>::new(krate.cid, u32::try_from(eager.list_expr.push(obj)).unwrap())
  }
}

impl<T: HirKind> GetApi<T> for Expr {
  fn get(krate: &Krate, id: HirId<T>) -> &Self {
    assert_eq!(id.cid(), krate.cid);
    match &krate.data {
      KrateData::Eager(eager) => &eager.list_expr[id.idx() as usize],
      KrateData::Lazy(lazy) => lazy.exprs[id.idx() as usize].get_or_init(|| lazy.loader.load_expr(id.idx() as usize)),
    }
  }
  fn get_mut(krate: &mut Krate, id: HirId<T>) -> &mut Self {
    assert_eq!(id.cid(), krate.cid);
    match &mut krate.data {
      KrateData::Eager(eager) => &mut eager.list_expr[id.idx() as usize],
      KrateData::Lazy(_) => panic!("cannot get_mut on lazy krate"),
    }
  }
}


impl<T: HirKind> PushApi<T> for Item {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> {
    let eager = match &mut krate.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot push to lazy krate"),
    };
    HirId::<T>::new(krate.cid, u32::try_from(eager.list_item.push(obj)).unwrap())
  }
}

impl<T: HirKind> GetApi<T> for Item {
  fn get(krate: &Krate, id: HirId<T>) -> &Self {
    assert_eq!(id.cid(), krate.cid);
    match &krate.data {
      KrateData::Eager(eager) => &eager.list_item[id.idx() as usize],
      KrateData::Lazy(lazy) => lazy.items[id.idx() as usize].get_or_init(|| lazy.loader.load_item(id.idx() as usize)),
    }
  }
  fn get_mut(krate: &mut Krate, id: HirId<T>) -> &mut Self {
    assert_eq!(id.cid(), krate.cid);
    match &mut krate.data {
      KrateData::Eager(eager) => &mut eager.list_item[id.idx() as usize],
      KrateData::Lazy(_) => panic!("cannot get_mut on lazy krate"),
    }
  }
}


impl<T: HirKind> PushApi<T> for Thing {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> {
    let eager = match &mut krate.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot push to lazy krate"),
    };
    HirId::<T>::new(krate.cid, u32::try_from(eager.list_thin.push(obj)).unwrap())
  }
}

impl<T: HirKind> GetApi<T> for Thing {
  fn get(krate: &Krate, id: HirId<T>) -> &Self {
    assert_eq!(id.cid(), krate.cid);
    match &krate.data {
      KrateData::Eager(eager) => &eager.list_thin[id.idx() as usize],
      KrateData::Lazy(lazy) => lazy.thins[id.idx() as usize].get_or_init(|| lazy.loader.load_thing(id.idx() as usize)),
    }
  }
  fn get_mut(krate: &mut Krate, id: HirId<T>) -> &mut Self {
    assert_eq!(id.cid(), krate.cid);
    match &mut krate.data {
      KrateData::Eager(eager) => &mut eager.list_thin[id.idx() as usize],
      KrateData::Lazy(_) => panic!("cannot get_mut on lazy krate"),
    }
  }
}


impl<T: HirKind> PushApi<T> for DefPath {
  fn push(krate: &mut Krate, obj: Self) -> HirId<T> {
    let eager = match &mut krate.data {
      KrateData::Eager(eager) => eager,
      KrateData::Lazy(_) => panic!("cannot push to lazy krate"),
    };
    HirId::<T>::new(krate.cid, u32::try_from(eager.list_dpath.push(obj)).unwrap())
  }
}

impl<T: HirKind> GetApi<T> for DefPath {
  fn get(krate: &Krate, id: HirId<T>) -> &Self {
    assert_eq!(id.cid(), krate.cid);
    match &krate.data {
      KrateData::Eager(eager) => &eager.list_dpath[id.idx() as usize],
      KrateData::Lazy(lazy) => lazy.dpaths[id.idx() as usize].get_or_init(|| lazy.loader.load_dpath(id.idx() as usize)),
    }
  }
  fn get_mut(krate: &mut Krate, id: HirId<T>) -> &mut Self {
    assert_eq!(id.cid(), krate.cid);
    match &mut krate.data {
      KrateData::Eager(eager) => &mut eager.list_dpath[id.idx() as usize],
      KrateData::Lazy(_) => panic!("cannot get_mut on lazy krate"),
    }
  }
}


// size api
pub trait SizeApi {
  fn size_used(cre: &Krate) -> usize;
  fn size_alloc(cre: &Krate) -> usize;
}

impl SizeApi for Type {
  fn size_used(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_type.len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.types.len(),
    }
  }
  fn size_alloc(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_type.allocated_len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.types.len(),
    }
  }
}

impl SizeApi for Expr {
  fn size_used(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_expr.len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.exprs.len(),
    }
  }
  fn size_alloc(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_expr.allocated_len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.exprs.len(),
    }
  }
}

impl SizeApi for Item {
  fn size_used(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_item.len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.items.len(),
    }
  }
  fn size_alloc(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_item.allocated_len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.items.len(),
    }
  }
}

impl SizeApi for Thing {
  fn size_used(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_thin.len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.thins.len(),
    }
  }
  fn size_alloc(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_thin.allocated_len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.thins.len(),
    }
  }
}

impl SizeApi for DefPath {
  fn size_used(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_dpath.len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.dpaths.len(),
    }
  }
  fn size_alloc(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => size_of::<Self>() * eager.list_dpath.allocated_len(),
      KrateData::Lazy(lazy) => size_of::<Self>() * lazy.dpaths.len(),
    }
  }
}

impl SizeApi for AnyId {
  fn size_used(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => (size_of::<HirId<SpecAny>>() * eager.extra_data.0.len()) + (size_of::<NodeKind>() * eager.extra_data.1.len()),
      KrateData::Lazy(lazy) => (size_of::<HirId<SpecAny>>() * lazy.extra_ids.len()) + (size_of::<NodeKind>() * lazy.extra_kinds.len()),
    }
  }
  fn size_alloc(cre: &Krate) -> usize {
    match &cre.data {
      KrateData::Eager(eager) => (size_of::<HirId<SpecAny>>() * eager.extra_data.0.allocated_len()) + (size_of::<NodeKind>() * eager.extra_data.1.allocated_len()),
      KrateData::Lazy(lazy) => (size_of::<HirId<SpecAny>>() * lazy.extra_ids.len()) + (size_of::<NodeKind>() * lazy.extra_kinds.len()),
    }
  }
}
