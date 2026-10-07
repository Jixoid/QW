/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#![feature(never_type)]

mod parse;
mod meta_p;
mod type_p;
mod expr_p;
mod item_p;
mod attr_p;
mod patt_p;
mod field_p;

type Fail<T> = Result<!, T>;

use parse::Ctx;
use {item_p::ItemParser, meta_p::{MetaParser, WordCheck}, attr_p::AttrParser, expr_p::ExprParser, type_p::TypeParser, patt_p::PattParser, field_p::FieldParser};

pub use parse::Parse;
