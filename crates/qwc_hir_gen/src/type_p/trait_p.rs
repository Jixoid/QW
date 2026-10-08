/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{hgen::Ctx, TypeLow};
use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_hir::{self as hir, PushOkApi, TypeAttr};


// Static Interface
pub fn low_trait(ctx: &mut Ctx, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::new_dsat(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()}.push(ctx.cre));

  let methods = {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

      let ast::FieldKind::Fun{kind, ..} = it.kind else { panic!() };

      let fun_ty = TypeLow::low(ctx, kind)?;
      
      
      // Post
      let id = hir::Thing::NamedType(
        it.name.unwrap().sid(),
        fun_ty
      ).push(ctx.cre);

      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };

  ctx.cmap.self_ty.pop();


  hir::Type {
    kind: hir::TypeKind::Trait(methods),
    layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
  }.push_ok(ctx.cre)
}


// Dynamic Inteface
pub fn low_iface(ctx: &mut Ctx, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::new_dsat(qwc_hir::LayoutBy::QW), attr: TypeAttr::empty()}.push(ctx.cre));

  let methods = {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

      let ast::FieldKind::Fun{kind, ..} = it.kind else { panic!() };
      let fun_ty = TypeLow::low(ctx, kind)?;
      
      // Check (self != DST)
      if let hir::TypeKind::Fun{self_kind: Some(self_kind), ..} = ctx.get_type(fun_ty).kind {

        let self_pos = if let ast::TypeKind::Fun{self_kind, ..} = ctx.src.get(kind).kind { ctx.src.get(self_kind.unwrap()).pos } else { unreachable!() }; 
    
        let lay = ctx.get_type(self_kind).layout;

        if !lay.is_static() {
          let pos = self_pos;
          return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["iface"]), Label::new_pos(pos)));
        }
      } else { panic!() }


      // Push
      let id = hir::Thing::NamedType(
        it.name.unwrap().sid(),
        fun_ty
      ).push(ctx.cre);

      ctn.push(id);
    }

    ctx.cre.extra(&ctn)
  };

  ctx.cmap.self_ty.pop();


  hir::Type {
    kind: hir::TypeKind::Iface(methods),
    layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
  }.push_ok(ctx.cre)
}
