use std::{marker::PhantomData, num::NonZero};

use serde::Serialize;

use crate::{Expr, Item, Type, krate::CID};


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
pub enum NodeKind { Any, Type, Expr, Item }


pub trait HirKind { fn kind() -> NodeKind; }

impl HirKind for SpecAny { fn kind() -> NodeKind { NodeKind::Any } }
impl HirKind for Type { fn kind() -> NodeKind { NodeKind::Type } }
impl HirKind for Expr { fn kind() -> NodeKind { NodeKind::Expr } }
impl HirKind for Item { fn kind() -> NodeKind { NodeKind::Item } }
