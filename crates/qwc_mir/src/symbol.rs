/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{BlokId, BlokRng, SymbRng, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
  Variable{ism: bool},
  Function{
    entry: BlokId,
    blocks: BlokRng,
  },
  Vmt {
    size: u64,
    align: u64,
    table: SymbRng,
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
