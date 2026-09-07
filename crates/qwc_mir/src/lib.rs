pub mod id;
pub mod dump;
mod symbol;
mod values;
mod blocks;
mod types;
mod krate;
mod ty_interner;

pub use krate::{Krate, Rng};
pub use {types::*, symbol::*, values::*, blocks::*};
pub use id::{AnyId, TypeId, SymbId, ValuId, BlokId};
pub use dump::Dump;
