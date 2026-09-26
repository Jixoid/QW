use std::{marker::PhantomData, num::NonZeroU32};

use crate::{Expr, Field, Item, Patt, Thing, Type};


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
pub type FieldId = AstId<Field>;
pub type ThingId = AstId<Thing>;



// Rng
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Rng<T>
  where T: AstKind
{
  start: u32,
  end: u32,
  pkind: PhantomData<T>,
}

impl<T: AstKind> Rng<T> {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end, pkind: PhantomData }
  }

  pub fn range(&self) -> std::ops::Range<usize> {
    (self.start as usize)..(self.end as usize)
  }

  pub fn empty() -> Self {
    Self{start: 0, end: 0, pkind: PhantomData}
  }

  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}

pub type TypeRng  = Rng<Type>;
pub type ExprRng  = Rng<Expr>;
pub type ItemRng  = Rng<Item>;
pub type PattRng  = Rng<Patt>;
pub type FieldRng = Rng<Field>;
pub type ThingRng = Rng<Thing>;



// AnyRng
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyRng {
  start: u32,
  end: u32,
}

impl AnyRng {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }

  pub fn range(&self) -> std::ops::Range<usize> {
    (self.start as usize)..(self.end as usize)
  }

  pub fn empty() -> Self {
    Self{start: 0, end: 0 }
  }

  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}



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
pub enum NodeKind { Any, Type, Expr, Item, Patt, Thing, Field }


pub trait AstKind { fn kind() -> NodeKind; }

impl AstKind for SpecAny { fn kind() -> NodeKind { NodeKind::Any } }
impl AstKind for Type  { fn kind() -> NodeKind { NodeKind::Type } }
impl AstKind for Expr  { fn kind() -> NodeKind { NodeKind::Expr } }
impl AstKind for Item  { fn kind() -> NodeKind { NodeKind::Item } }
impl AstKind for Patt  { fn kind() -> NodeKind { NodeKind::Patt } }
impl AstKind for Field { fn kind() -> NodeKind { NodeKind::Field } }
impl AstKind for Thing { fn kind() -> NodeKind { NodeKind::Thing } }
