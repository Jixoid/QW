use crate::{BlokId, TypeId, id::ValuId};


#[derive(Debug, Copy, Clone)]
pub enum SymbolKind {
  Variable{ism: bool, value: ValuId},
  Function{blok: BlokId},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SymbolStat {
  Private, // Private, only in module
  Normal, // Inter-Modules
  Export,
  Import,
}


#[derive(Debug, Copy, Clone)]
pub struct Symbol {
  pub name: u32,
  pub kind: SymbolKind,
  pub stat: SymbolStat,
  pub ety: TypeId,
}  
