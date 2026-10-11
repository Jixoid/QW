/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_hir::{self as hir, DefPathId, TypeAttr};

use crate::{Ctx, TypeLow, type_p::read_attrs};


// Combinated
pub fn low_struct(ctx: &mut Ctx, path: DefPathId, id: ast::TypeId, rng: ast::ThingRng) -> Result<hir::TypeId, Message> {
  let fields_rng = {
    let mut ctn = vec![];
    
    for id in ctx.src.extra_get(rng) {
      let ast::ThingKind::NamedType(name, kind) = ctx.src.get(id).kind else { panic!() };

      // Check
      let id = TypeLow::low(ctx, kind)?;
      let lay = ctx.get(id).layout;

      if lay.is_dynamic() {
        return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["tuple"]), Label::new_pos(name)))
      }
      
      let id = ctx.cre.push(hir::Thing::NamedType(name.sid(), id));
      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };

  let attr = read_attrs(ctx, ctx.src.get_attached(id))?;

  let layout = hir::Layout::new_static(if attr.contains(TypeAttr::C) {hir::LayoutBy::QW} else {hir::LayoutBy::C});


  // Post
  let this = hir::Type {
    kind: hir::TypeKind::Struct(fields_rng),
    layout,
    attr,
    path,
  };

  Ok(ctx.cre.push(this))
}

pub fn low_tuple(ctx: &mut Ctx, path: DefPathId, rng: ast::TypeRng) -> Result<hir::TypeId, Message> {
  let rng = {
    let mut ctn = vec![];
    
    for id in ctx.src.extra_get(rng) {
      // Check
      let pos = ctx.src.get(id).pos;
      let id = TypeLow::low(ctx, id)?;
      let lay = ctx.get(id).layout;

      if lay.is_dynamic() {
        return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["tuple"]), Label::new_pos(pos)))
      }
      
      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };


  // Post
  let this = hir::Type {
    kind: hir::TypeKind::Tuple(rng),
    layout: hir::Layout::new_static(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
    path,
  };
  
  Ok(ctx.cre.push(this))
}
