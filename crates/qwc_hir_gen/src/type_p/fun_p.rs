/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{self as hir, DefPathId, PushOkApi, TypeAttr};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_ast as ast;

use crate::{Ctx, TypeLow};


// Function
pub fn low_fun(ctx: &mut Ctx, path: DefPathId, self_kind: Option<ast::TypeId>, rng: ast::ThingRng, ret: Option<ast::TypeId>) -> Result<hir::TypeId, Message> {
  let args = {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let ast::ThingKind::NamedType(name, kind) = ctx.src.get(id).kind else { panic!() };

      let id = hir::Thing::NamedType(name.sid(), TypeLow::low(ctx, kind)?).push(ctx.cre);
      
      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };

  if let Some(self_kind) = self_kind && ctx.cmap.self_ty.last().is_none() {
    let self_kind = ctx.src.get(self_kind);
    
    return Err(Message::error(SELF_PARAMETER_IS_ONLY_ALLOWED_IN_ASSOCIATED_FUN, Label::new_pos(self_kind.pos)))
  }

  let self_kind = self_kind.map(|id| TypeLow::low(ctx, id)).transpose()?;

  let ret = match ret {
    None => ctx.prims.ty_unit,
    Some(kind) => TypeLow::low(ctx, kind)?,
  };


  // Post
  let this = hir::Type {
    kind: hir::TypeKind::Fun{self_kind, args, ret},
    layout: hir::Layout::new_dst(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
    path,
  };
  
  Ok(ctx.cre.push(this))
}
