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
use qwc_hir::{self as hir, DefPathId, PushOkApi, TypeAttr};


// Static Interface
pub fn low_trait(ctx: &mut Ctx, path: DefPathId, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
  let trait_ty = hir::Type {
    kind: hir::TypeKind::Error,
    layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
    path,
  }.push(ctx.cre);

  ctx.cmap.current_trait.push(trait_ty);
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::new_dsat(qwc_hir::LayoutBy::QW), path, attr: TypeAttr::empty()}.push(ctx.cre));

  let (methods, types) = {
    let mut methods = vec![];
    let mut types = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

      match it.kind {
        ast::FieldKind::Fun{kind, ..} => {
          let name = it.name.unwrap().sid();
          let fun_path = ctx.cre.push(hir::DefPath::Path { base: path, name });
          let fun_ty = TypeLow::low_itemty(ctx, fun_path, kind)?;

          // Post
          let id = hir::Thing::NamedType(
            name,
            fun_ty
          ).push(ctx.cre);
    
          methods.push(id);
        }

        ast::FieldKind::Type(kind) => {
          let kind = kind.map(|id| TypeLow::low(ctx, id)).transpose()?;

          // Post
          let id = match kind {
            Some(kind) => hir::Thing::NamedType(it.name.unwrap().sid(), kind),
            
            None => hir::Thing::Name(it.name.unwrap().sid()),
          }.push(ctx.cre) as hir::ThingId;

          types.push(id);
        }
        
        _ => todo!("{it:#?}")
      }
    }

    (ctx.cre.extra(&methods), ctx.cre.extra(&types))
  };

  ctx.cmap.self_ty.pop();
  ctx.cmap.current_trait.pop();

  let this = hir::Type {
    kind: hir::TypeKind::Trait(methods, types),
    layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
    path,
  };
  *ctx.cre.get_mut(trait_ty) = this;

  Ok(trait_ty)
}


// Dynamic Inteface
pub fn low_iface(ctx: &mut Ctx, path: DefPathId, rng: ast::FieldRng) -> Result<hir::TypeId, Message> {
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::new_dsat(qwc_hir::LayoutBy::QW), path, attr: TypeAttr::empty()}.push(ctx.cre));

  let methods = {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

      let ast::FieldKind::Fun{kind, ..} = it.kind else { unreachable!() };
      let name = it.name.unwrap().sid();
      let fun_path = ctx.cre.push(hir::DefPath::Path { base: path, name });
      let fun_ty = TypeLow::low_itemty(ctx, fun_path, kind)?;
      
      // Check (self != DST)
      let hir::TypeKind::Fun{self_kind: Some(self_kind), ..} = ctx.get(fun_ty).kind else { unreachable!() };

      let self_pos = if let ast::TypeKind::Fun{self_kind, ..} = ctx.src.get(kind).kind { ctx.src.get(self_kind.unwrap()).pos } else { unreachable!() }; 
  
      let lay = ctx.get(self_kind).layout;

      if !lay.is_static() {
        let pos = self_pos;
        return Err(Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["iface"]), Label::new_pos(pos)));
      }


      // Push
      let id = hir::Thing::NamedType(
        name,
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
    path,
  }.push_ok(ctx.cre)
}
