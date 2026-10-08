/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


mod cgen;
mod value_p;
mod fun_ctx;
mod symb_p;
mod inst_p;
mod blok_p;
mod type_p;

use cgen::{CtxI, CtxM};
use fun_ctx::FunCtx;
use symb_p::SymbLow;
use blok_p::BlokLow;
use inst_p::InstLow;
use type_p::TypeLow;
use value_p::ValueLow;

pub use cgen::CGenLLVM as CGen;
