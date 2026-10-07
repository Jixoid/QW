/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::Itertools;
use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast as ast;
use qwc_hir::{self as hir, PushOkApi};
use rustc_hash::FxHashMap;

use crate::Ctx;


pub fn low_enum(ctx: &mut Ctx, rng: ast::ThingRng) -> Result<hir::TypeId, Message> {
  let mut names = FxHashMap::default();
  let mut enum_val: i128 = 0;

  let rng = ctx.src.extra_get(rng).map(|id| {
    let (name, _expr) = match *ctx.src.get(id) {
      ast::Thing::Name(name) => (name, None),
      ast::Thing::NamedExpr(name, expr) => (name, Some(expr)),
      _ => unreachable!()
    };

    use std::collections::hash_map::Entry::*;

    // Duplicate Test
    match names.entry(name.sid()) {
      Occupied(entry) => {
        ctx.sum.add(Message::error(DUPLICATE_IDENTIFIER, Label::new(name, CONFLICTING_DEFINITION))
          .add(Label::new(*entry.get(), FIRST_DEFINITION_HERE))
        );
      }

      Vacant(entry) => {
        entry.insert(Into::<Span>::into(name));
      }
    };
    

    let id = hir::Thing::NamedConst(name.sid(), hir::Const::Int(enum_val as i32)).push(ctx.cre);

    enum_val += 1;

    id
  }).collect_vec();

  let rng = ctx.cre.extra(&rng);
  

  // Post
  hir::Type {
    kind: hir::TypeKind::Enum(rng),
    layout: hir::Layout::new_static(hir::LayoutBy::QW),
  }.push_ok(ctx.cre)
}
