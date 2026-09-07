use std::{marker::PhantomData, num::NonZeroU32};

use crate::{Block, Symbol, Type, Value};


// AstId
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MirId<T>
where T: MirKind
{
  idx: NonZeroU32,
  pkind: PhantomData<T>
}


impl<T: MirKind> MirId<T> {

  pub(crate) fn new(idx: u32) -> Self {
    Self{idx: NonZeroU32::new(idx+1).unwrap(), pkind: PhantomData}
  }

  pub fn new_from(id: (MirId<SpecAny>, NodeKind)) -> Self {
    assert_eq!(T::kind(), id.1);
    MirId::<T>::new(id.0.idx())
  }

  pub(crate) fn idx(&self) -> u32 {
    self.idx.get()-1
  }


  pub fn to_any(&self) -> AnyId {
    AnyId::new(self.idx(), T::kind())
  }

  pub fn from_any(id: AnyId) -> Self {
    assert_eq!(T::kind(), id.kind());
    MirId::<T>::new(id.idx())
  }

}

impl<T: MirKind> Into<AnyId> for MirId<T> {
  fn into(self) -> AnyId { self.to_any() }
}

pub type TypeId  = MirId<Type>;
pub type SymbId  = MirId<Symbol>;
pub type ValuId  = MirId<Value>;
pub type BlokId  = MirId<Block>;


// AnyId
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct SpecAny;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct AnyId {
  id: MirId<SpecAny>,
  kind: NodeKind,
}

impl AnyId {

  pub fn new(idx: u32, kind: NodeKind) -> Self {
    Self{ id: MirId::new(idx), kind }
  }

  pub fn new_from(id: (MirId<SpecAny>, NodeKind)) -> Self {
    Self { id: id.0, kind: id.1 }
  }


  pub(crate) fn idx(&self) -> u32 {
    self.id.idx()
  }

  pub(crate) fn id(&self) -> MirId<SpecAny> {
    self.id
  }

  pub(crate) fn kind(&self) -> NodeKind {
    self.kind
  }

}


// Trait
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind { Any, Type, Symb, Value, Blok }


pub trait MirKind { fn kind() -> NodeKind; }

impl MirKind for SpecAny { fn kind() -> NodeKind { NodeKind::Any } }
impl MirKind for Type   { fn kind() -> NodeKind { NodeKind::Type } }
impl MirKind for Symbol { fn kind() -> NodeKind { NodeKind::Symb } }
impl MirKind for Value  { fn kind() -> NodeKind { NodeKind::Value } }
impl MirKind for Block  { fn kind() -> NodeKind { NodeKind::Blok } }
