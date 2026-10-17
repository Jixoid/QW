/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#![feature(char_to_u32)]

mod mgen;
mod layout;
mod symb_p;
mod type_p;
mod blok_p;
mod expr_p;
mod builder;
mod ty_interner;

use mgen::Ctx;
use {symb_p::*, type_p::*, blok_p::*, expr_p::*};
use builder::*;
use layout::Layouter;

pub use mgen::{MGen, CacheMap};
