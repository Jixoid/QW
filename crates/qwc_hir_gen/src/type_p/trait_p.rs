/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{hgen::Ctx, TypeLow};
use qwc_ast as ast;
use qwc_diagnostic::{Label, Message, Status::{self, Fail, Success}, StatusApi, Summary, msg::*, ret_err, send_err};
use qwc_hir::{self as hir, DefPathId, PushOkApi, TypeAttr};


// Static Interface
pub fn low_trait(ctx: &mut Ctx, sum: &mut Summary, path: DefPathId, rng: ast::FieldRng, reqs: ast::TypeRng) -> Status<hir::TypeId> {
  let trait_ty = hir::Type {
    kind: hir::TypeKind::Error,
    layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
    path,
  }.push(ctx.cre);

  let mut has_error = false;
  let mut req_vec = vec![];

  for id in ctx.src.extra_get(reqs) {
    let id_hir = match TypeLow::low(ctx, sum, id) {
      Success(id_hir) => id_hir,
      Fail => {
        has_error = true;
        continue;
      }
    };

    match ctx.get(id_hir).kind {
      hir::TypeKind::Iface(..) | hir::TypeKind::Trait{..} => {
        req_vec.push(id_hir);
      }
      hir::TypeKind::GenericRaw{..} => {
        has_error = true;
        send_err!(sum, Message::error(CANNOT_USE_GENERIC_WITHOUT_SPECIALIZATION, Label::new_pos(ctx.src.get(id).pos)));
      }
      hir::TypeKind::Error => {
        has_error = true;
      }
      _ => {
        has_error = true;
        send_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["trait", &ctx.type_name(sum, trait_ty)]), Label::new_pos(ctx.src.get(id).pos)));
      }
    }
  }

  if has_error {
    return Status::Success(trait_ty);
  }

  let reqs = ctx.cre.extra(&req_vec);

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
          let fun_ty = TypeLow::low_itemty(ctx, sum, fun_path, kind)?;

          // Post
          let id = hir::Thing::NamedType(
            name,
            fun_ty
          ).push(ctx.cre);
    
          methods.push(id);
        }

        ast::FieldKind::Type(kind) => {
          let kind = kind.map(|id| TypeLow::low(ctx, sum, id)).transpose()?;

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
    kind: hir::TypeKind::Trait{methods, types, reqs},
    layout: hir::Layout::new_dsat(hir::LayoutBy::QW),
    attr: TypeAttr::empty(),
    path,
  };
  *ctx.cre.get_mut(trait_ty) = this;

  Status::Success(trait_ty)
}


// Dynamic Inteface
pub fn low_iface(ctx: &mut Ctx, sum: &mut Summary, path: DefPathId, rng: ast::FieldRng) -> Status<hir::TypeId> {
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::new_dsat(qwc_hir::LayoutBy::QW), path, attr: TypeAttr::empty()}.push(ctx.cre));

  let methods = {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

      let ast::FieldKind::Fun{kind, ..} = it.kind else { unreachable!() };
      let name = it.name.unwrap().sid();
      let fun_path = ctx.cre.push(hir::DefPath::Path { base: path, name });
      let fun_ty = TypeLow::low_itemty(ctx, sum, fun_path, kind)?;
      
      // Check (self != DST)
      let hir::TypeKind::Fun{self_kind: Some(self_kind), ..} = ctx.get(fun_ty).kind else { unreachable!() };

      let self_pos = if let ast::TypeKind::Fun{self_kind, ..} = ctx.src.get(kind).kind { ctx.src.get(self_kind.unwrap()).pos } else { unreachable!() }; 
  
      let lay = ctx.get(self_kind).layout;

      if !lay.is_static() {
        let pos = self_pos;
        ret_err!(sum, Message::error(DST_TYPES_CANNOT_EXIST_IN_X.args(&["iface"]), Label::new_pos(pos)));
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
