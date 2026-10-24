/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, PushOkApi, TypeAttr};

use crate::{Ctx, TypeLow, type_p::helper};


// Combinated
pub fn low_struct(ctx: &mut Ctx, sum: &mut Summary, id: ast::TypeId, path: ast::DefPathId, vars: ast::ThingRng) -> Status<hir::TypeId> {
  let vars = {
    let mut ctn = vec![];
    
    for id in ctx.src.extra_get(vars) {
      let ast::ThingKind::NamedType(name, kind) = ctx.src.get(id).kind else { unreachable!() };

      // Check
      let id = TypeLow::low(ctx, sum, kind)?;
      let lay = ctx.get(id).layout;

      if lay.is_dynamic() {
        ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["tuple"]), Label::new_pos(name)));
      }
      
      let id = ctx.cre.push(hir::Thing::NamedType(name.sid(), id));
      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };

  let path = helper::low_defpath(ctx, path);
  let attr = helper::read_attrs(ctx, sum, ctx.src.get_attached(id))?;


  // Post
  hir::Type {
    kind: hir::TypeKind::ItemStruct{fields: vars, path},
    layout: hir::Layout::STATIC,
    attr,
  }.push_ok(ctx.cre)
}

pub fn low_tuple(ctx: &mut Ctx, sum: &mut Summary, path: ast::DefPathId, vars: ast::TypeRng) -> Status<hir::TypeId> {
  let vars = {
    let mut ctn = vec![];
    
    for id in ctx.src.extra_get(vars) {
      // Check
      let pos = ctx.src.get(id).pos;
      let id = TypeLow::low(ctx, sum, id)?;
      let lay = ctx.get(id).layout;

      if lay.is_dynamic() {
        ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["tuple"]), Label::new_pos(pos)))
      }
      
      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };

  let path = helper::low_defpath(ctx, path);


  // Post
  hir::Type {
    kind: hir::TypeKind::ItemTuple{fields: vars, path},
    layout: hir::Layout::STATIC,
    attr: TypeAttr::empty(),
  }.push_ok(ctx.cre)
}


pub fn low_tuple_anon(ctx: &mut Ctx, sum: &mut Summary, rng: ast::TypeRng) -> Status<hir::TypeId> {
  let elem_ids = {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let elem_ty = TypeLow::low(ctx, sum, id)?;
      ctn.push(elem_ty);
    }
    
    ctx.cre.extra(&ctn)
  };
  

  // Post
  hir::Type {
    kind: hir::TypeKind::Tuple(elem_ids),
    layout: hir::Layout::STATIC,
    attr: TypeAttr::empty(),
  }.push_ok(ctx.cre)
}
