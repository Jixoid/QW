use std::{marker::PhantomData, num::NonZeroU32};

use crate::{Expr, Item, Patt, Thing, Type};


// AstId
#[derive(Debug, Copy, Clone)]
pub struct AstId<T>
where T: AstKind
{
  idx: NonZeroU32,
  pkind: PhantomData<T>
}

impl<T: AstKind> PartialEq for AstId<T> {
  fn eq(&self, other: &Self) -> bool {
    self.idx == other.idx
  }
}

impl<T: AstKind> Eq for AstId<T> {}

impl<T: AstKind> std::hash::Hash for AstId<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.idx.hash(state);
  }
}


impl<T: AstKind> AstId<T> {

  pub(crate) fn new(idx: u32) -> Self {
    Self{idx: NonZeroU32::new(idx+1).unwrap(), pkind: PhantomData}
  }

  pub fn new_from(id: (AstId<SpecAny>, NodeKind)) -> Self {
    assert_eq!(T::kind(), id.1);
    AstId::<T>::new(id.0.idx())
  }

  pub(crate) fn idx(&self) -> u32 {
    self.idx.get()-1
  }


  pub fn to_any(&self) -> AnyId {
    AnyId::new(self.idx(), T::kind())
  }

  pub fn from_any(id: AnyId) -> Self {
    assert_eq!(T::kind(), id.kind());
    AstId::<T>::new(id.idx())
  }

}

impl<T: AstKind> Into<AnyId> for AstId<T> {
  fn into(self) -> AnyId { self.to_any() }
}

pub type TypeId  = AstId<Type>;
pub type ExprId  = AstId<Expr>;
pub type ItemId  = AstId<Item>;
pub type PattId  = AstId<Patt>;
pub type ThingId = AstId<Thing>;


// AnyId
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct SpecAny;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyId {
  id: AstId<SpecAny>,
  kind: NodeKind,
}

impl AnyId {

  pub fn new(idx: u32, kind: NodeKind) -> Self {
    Self{ id: AstId::new(idx), kind }
  }

  pub fn new_from(id: (AstId<SpecAny>, NodeKind)) -> Self {
    Self { id: id.0, kind: id.1 }
  }


  pub(crate) fn idx(&self) -> u32 {
    self.id.idx()
  }

  pub(crate) fn id(&self) -> AstId<SpecAny> {
    self.id
  }

  pub(crate) fn kind(&self) -> NodeKind {
    self.kind
  }

}


// Trait
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind { Any, Type, Expr, Item, Patt, Thing }


pub trait AstKind { fn kind() -> NodeKind; }

impl AstKind for SpecAny { fn kind() -> NodeKind { NodeKind::Any } }
impl AstKind for Type { fn kind() -> NodeKind { NodeKind::Type } }
impl AstKind for Expr { fn kind() -> NodeKind { NodeKind::Expr } }
impl AstKind for Item { fn kind() -> NodeKind { NodeKind::Item } }
impl AstKind for Patt { fn kind() -> NodeKind { NodeKind::Patt } }
impl AstKind for Thing { fn kind() -> NodeKind { NodeKind::Thing } }
