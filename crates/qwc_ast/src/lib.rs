/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


mod krate;
mod dump;
mod ident;
mod attrs;
mod items;
mod types;
mod exprs;
mod patts;
mod things;
mod fields;
pub mod id;
mod visitor;

pub use krate::{Krate, IdentSave, PushApi, GetApi, AttachNode};
pub use id::{AnyId, TypeId, ExprId, ItemId, FieldId, ThingId, PattId, Rng, AnyRng, TypeRng, ExprRng, ItemRng, FieldRng, ThingRng, PattRng};
pub use {ident::Ident, attrs::Attribute, attrs::AttrKind, dump::Dump};
pub use {items::*, fields::*, types::*, exprs::*, patts::*, things::*};
pub use visitor::Visitor;
