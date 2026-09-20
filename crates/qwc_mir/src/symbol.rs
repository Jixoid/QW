use crate::{BlokId, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  Variable{ism: bool},
  Function{blok: BlokId},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymbolStat {
  Private, // Private, only in module
  Normal, // Inter-Modules
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
