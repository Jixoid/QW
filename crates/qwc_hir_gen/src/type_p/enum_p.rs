/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::Itertools;
use qwc_diagnostic::{Label, Message, Span, Status, Summary, msg::*};
use qwc_ast as ast;
use qwc_hir::{self as hir, PushOkApi, TypeAttr};
use rustc_hash::FxHashMap;

use crate::{Ctx, type_p::helper};


pub fn low_enum(ctx: &mut Ctx, sum: &mut Summary, path: ast::DefPathId, rng: ast::ThingRng) -> Status<hir::TypeId> {
  let mut names = FxHashMap::default();
  let mut enum_val: i128 = 0;

  let rng = ctx.src.extra_get(rng).map(|id| {
    let (name, _) = match ctx.src.get(id).kind {
      ast::ThingKind::Name(name) => (name, None),
      ast::ThingKind::NamedExpr(name, expr) => (name, Some(expr)),
      _ => unreachable!()
    };

    use std::collections::hash_map::Entry::*;

    // Duplicate Test
    match names.entry(name.sid()) {
      Occupied(entry) => {
        sum.add(Message::error(DUPLICATE_IDENTIFIER, Label::new(name, CONFLICTING_DEFINITION))
          + Label::new(*entry.get(), FIRST_DEFINITION_HERE)
        );
      }

      Vacant(entry) => {
        entry.insert(Into::<Span>::into(name));
      }
    };
    

    let id = hir::Thing::NamedConst(name.sid(), hir::Const::I32(enum_val as i32)).push(ctx.cre);

    enum_val += 1;

    id
  }).collect_vec();

  let path = helper::low_defpath(ctx, path);

  let rng = ctx.cre.extra(&rng);
  

  // Post
  hir::Type {
    kind: hir::TypeKind::ItemEnum{values: rng, path},
    layout: hir::Layout::STATIC,
    attr: TypeAttr::empty(),
  }.push_ok(ctx.cre)
}

pub fn low_flags(ctx: &mut Ctx, sum: &mut Summary, path: ast::DefPathId, rng: ast::ThingRng) -> Status<hir::TypeId> {
  let mut names = FxHashMap::default();
  let mut flag_val: i128 = 0;

  let rng = ctx.src.extra_get(rng).map(|id| {
    let (name, _) = match ctx.src.get(id).kind {
      ast::ThingKind::Name(name) => (name, None),
      ast::ThingKind::NamedExpr(name, expr) => (name, Some(expr)),
      _ => unreachable!()
    };

    use std::collections::hash_map::Entry::*;

    // Duplicate Test
    match names.entry(name.sid()) {
      Occupied(entry) => {
        sum.add(Message::error(DUPLICATE_IDENTIFIER, Label::new(name, CONFLICTING_DEFINITION))
          + Label::new(*entry.get(), FIRST_DEFINITION_HERE)
        );
      }

      Vacant(entry) => {
        entry.insert(Into::<Span>::into(name));
      }
    };
    

    let id = hir::Thing::NamedConst(name.sid(), hir::Const::I32(flag_val as i32)).push(ctx.cre);

    flag_val <<= 1;

    id
  }).collect_vec();

  let path = helper::low_defpath(ctx, path);

  let rng = ctx.cre.extra(&rng);
  

  // Post
  hir::Type {
    kind: hir::TypeKind::ItemFlag{values: rng, path},
    layout: hir::Layout::STATIC,
    attr: TypeAttr::empty(),
  }.push_ok(ctx.cre)
}
