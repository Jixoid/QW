/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


mod hgen;
mod type_p;
mod expr_p;
mod item_p;
mod specialize;
mod ty_interner;

use specialize::Specialization;
use hgen::{Ctx, FunCtx};
use {type_p::{TypeLow, TypeMatch}, expr_p::ExprLow, item_p::ItemLow};

pub use hgen::HGen;
