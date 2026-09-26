use crate::{BlokId, Rng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  Variable{ism: bool},
  Function{
    entry: BlokId,
    blocks: Rng, /* BlokId */
    stack: Rng,  /* TypeId */
  },
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymbolStat {
  Private, // Private, only in module
  Internal, // Inter-Modules
  Export,
  Import,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
  pub name: u32,
  pub kind: SymbolKind,
  pub stat: SymbolStat,
  pub ety: TypeId,
}  
