/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::NonZeroU32;

use crate::{Layout, TypeId, TypeRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // NST
  Unit,
  
  // Primitive
  Int(NonZeroU32, bool),
  Float(FloatKind),
  
  Bool,
  Ptr,

  // Combinated
  Struct(TypeRng),
  
  // Sequential
  Array(TypeId, u32),
  Slice(TypeId),
  
  // Callable
  Fun{args: TypeRng, ret: TypeId},
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub kind: TypeKind,
  pub layout: Layout,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FloatKind {
  BF16,
  F16,
  F32,
  F64,
  F128,
}
