/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{marker::PhantomData, num::NonZero};

use serde::Serialize;

use crate::{Expr, Item, Krate, PushApi, Thing, Type, krate::CID};


// AstId
#[derive(Serialize, Debug, Copy, Clone)]
pub struct HirId<T>
  where T: HirKind
{
  cid: CID,
  idx: NonZero<u32>,
  
  #[serde(skip)]
  pkind: PhantomData<T>
}


impl<T: HirKind> PartialEq for HirId<T> {
  fn eq(&self, other: &Self) -> bool {
    self.cid == other.cid && self.idx == other.idx
  }
}

impl<T: HirKind> Eq for HirId<T> {}

impl<T: HirKind> std::hash::Hash for HirId<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.cid.hash(state);
    self.idx.hash(state);
  }
}


impl<T: HirKind> HirId<T> {

  pub(crate) fn new(cid: CID, idx: u32) -> Self {
    Self{cid, idx: NonZero::<u32>::new(idx+1).unwrap(), pkind: PhantomData}
  }

  pub fn new_from(id: (HirId<SpecAny>, NodeKind)) -> Self {
    assert_eq!(T::kind(), id.1);
    HirId::<T>::new(id.0.cid(), id.0.idx())
  }


  pub(crate) fn idx(&self) -> u32 {
    self.idx.get()-1
  }

  pub fn cid(&self) -> CID {
    self.cid
  }


  pub fn to_any(&self) -> AnyId {
    AnyId::new(self.cid, self.idx(), T::kind())
  }

  pub fn from_any(id: AnyId) -> Self {
    assert_eq!(T::kind(), id.kind());
    HirId::<T>::new(id.cid(), id.idx())
  }

}

impl<T: HirKind> Into<AnyId> for HirId<T> {
  fn into(self) -> AnyId { self.to_any() }
}

pub type TypeId  = HirId<Type>;
pub type ExprId  = HirId<Expr>;
pub type ItemId  = HirId<Item>;
pub type ThingId = HirId<Thing>;



// Rng
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Rng<T>
  where T: HirKind
{
  cid: CID,
  start: u32,
  end: u32,
  pkind: PhantomData<T>,
}

impl<T: HirKind> Rng<T> {
  pub(crate) fn new(cid: CID, start: u32, end: u32) -> Self {
    Self { cid, start, end, pkind: PhantomData }
  }

  pub fn count(&self) -> u32 {
    self.end - self.start
  }

  pub fn range(&self) -> std::ops::Range<usize> {
    (self.start as usize)..(self.end as usize)
  }

  pub fn cid(&self) -> CID {
    self.cid
  }

  pub fn empty(cid: CID) -> Self {
    Self{cid, start: 0, end: 0, pkind: PhantomData}
  }

  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}

pub type TypeRng  = Rng<Type>;
pub type ExprRng  = Rng<Expr>;
pub type ItemRng  = Rng<Item>;
pub type ThingRng = Rng<Thing>;



// AnyRng
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyRng {
  cid: CID,
  start: u32,
  end: u32,
}

impl AnyRng {
  pub(crate) fn new(cid: CID, start: u32, end: u32) -> Self {
    Self { cid, start, end }
  }

  pub fn range(&self) -> std::ops::Range<usize> {
    (self.start as usize)..(self.end as usize)
  }

  pub fn cid(&self) -> CID {
    self.cid
  }

  pub fn empty(cid: CID) -> Self {
    Self{cid, start: 0, end: 0 }
  }

  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}



// PushOkApi
pub trait PushOkApi<T: HirKind> {
  fn push(self, cre: &mut Krate) -> HirId<T>;
  fn push_ok<E>(self, cre: &mut Krate) -> Result<HirId<T>, E>;
}

impl<T: HirKind, P: PushApi<T>> PushOkApi<T> for P {
  fn push(self, cre: &mut Krate) -> HirId<T> {
    cre.push(self)
  }
  
  fn push_ok<E>(self, cre: &mut Krate) -> Result<HirId<T>, E> {
    Ok(cre.push(self))
  }
}



// AnyId
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct SpecAny;

#[derive(Serialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyId {
  cid: CID,
  idx: NonZero<u32>,
  kind: NodeKind,
}

impl AnyId {

  pub fn new(cid: CID, idx: u32, kind: NodeKind) -> Self {
    Self{ cid, idx: NonZero::<u32>::new(idx+1).unwrap(), kind }
  }

  pub fn new_from(id: (HirId<SpecAny>, NodeKind)) -> Self {
    Self { cid: id.0.cid, idx: id.0.idx, kind: id.1 }
  }


  pub(crate) fn idx(&self) -> u32 {
    self.idx.get()-1
  }

  pub fn cid(&self) -> CID {
    self.cid
  }


  pub(crate) fn id(&self) -> HirId<SpecAny> {
    HirId { cid: self.cid, idx: self.idx, pkind: PhantomData }
  }

  pub(crate) fn kind(&self) -> NodeKind {
    self.kind
  }

}


// Trait
#[derive(Serialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind { Any, Type, Expr, Item, Thing }


pub trait HirKind { fn kind() -> NodeKind; }

impl HirKind for SpecAny { fn kind() -> NodeKind { NodeKind::Any } }
impl HirKind for Type  { fn kind() -> NodeKind { NodeKind::Type } }
impl HirKind for Expr  { fn kind() -> NodeKind { NodeKind::Expr } }
impl HirKind for Item  { fn kind() -> NodeKind { NodeKind::Item } }
impl HirKind for Thing { fn kind() -> NodeKind { NodeKind::Thing } }
