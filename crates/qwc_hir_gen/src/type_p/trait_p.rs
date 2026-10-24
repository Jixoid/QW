/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{TypeLow, Ctx, type_p::helper};
use qwc_ast as ast;
use qwc_diagnostic::{CollectOrErrors, Label, Message, Status::{self, Success}, StatusApi, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, PushOkApi, TypeAttr};


// Static Interface
pub fn low_trait(ctx: &mut Ctx, sum: &mut Summary, path: ast::DefPathId, fields: ast::FieldRng, reqs: ast::TypeRng) -> Status<hir::TypeId> {
  // Reqs
  let reqs = ctx.src.extra_get(reqs).map(|id| {
    let id_hir = TypeLow::low(ctx, sum, id)?;
    
    match ctx.get(id_hir).kind {
      hir::TypeKind::ItemIface{..} | hir::TypeKind::ItemTrait{..} => {}
      hir::TypeKind::Error => {}
      
      hir::TypeKind::GenericRaw{..} => ret_err!(sum, Message::error(CANNOT_USE_GENERIC_WITHOUT_SPECIALIZATION, Label::new_pos(ctx.src.get(id).pos))),
      
      _ => ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["trait", &ctx.type_name(sum, id_hir)]), Label::new_pos(ctx.src.get(id).pos))),
    };

    Success(id_hir)
  })
    .collect_or_errors()?;
  
  let reqs = ctx.cre.extra(&reqs);


  // Dummy
  let path = helper::low_defpath(ctx, path);

  let dummy_me = hir::Type {
    kind: hir::TypeKind::Error,
    attr: TypeAttr::empty(),
    layout: hir::Layout::DSAT,
  }.push(ctx.cre);


  ctx.cmap.current_trait.push(dummy_me);
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::DSAT, attr: TypeAttr::empty()}.push(ctx.cre));


  // fields
  let fields = (|| {
    let mut methods = vec![];
    let mut types = vec![];

    for id in ctx.src.extra_get(fields) {
      let it = ctx.src.get(id);

      match it.kind {
        ast::FieldKind::Fun{kind, ..} => {
          let name = it.name.unwrap().sid();
          let fun_ty = TypeLow::low(ctx, sum, kind)?;

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

    Success((ctx.cre.extra(&methods), ctx.cre.extra(&types)))
  })();

  ctx.cmap.self_ty.pop();
  ctx.cmap.current_trait.pop();


  let (methods, types) = fields?;

  let this = hir::Type {
    kind: hir::TypeKind::ItemTrait{methods, types, reqs, path},
    layout: hir::Layout::DSAT,
    attr: TypeAttr::empty(),
  };
  *ctx.cre.get_mut(dummy_me) = this;

  Status::Success(dummy_me)
}


// Dynamic Inteface
pub fn low_iface(ctx: &mut Ctx, sum: &mut Summary, path: ast::DefPathId, rng: ast::FieldRng, reqs: ast::TypeRng) -> Status<hir::TypeId> {
  // Dummy
  let path = helper::low_defpath(ctx, path);
  
  let dummy_me = hir::Type {
    kind: hir::TypeKind::Error,
    attr: TypeAttr::empty(),
    layout: hir::Layout::DSAT,
  }.push(ctx.cre);

  ctx.cmap.current_trait.push(dummy_me);
  ctx.cmap.self_ty.push(hir::Type{kind: hir::TypeKind::GenericSelfType, layout: hir::Layout::DSAT, attr: TypeAttr::empty()}.push(ctx.cre));


  let methods = (|| {
    let mut ctn = vec![];

    for id in ctx.src.extra_get(rng) {
      let it = ctx.src.get(id);

      let ast::FieldKind::Fun{kind, ..} = it.kind else { unreachable!() };
      let name = it.name.unwrap().sid();
      let fun_ty = TypeLow::low(ctx, sum, kind)?;
      
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

    Success(ctx.cre.extra(&ctn))
  })();

  ctx.cmap.self_ty.pop();

  
  // Post
  let this = hir::Type {
    kind: hir::TypeKind::ItemIface{methods: methods?, path},
    layout: hir::Layout::DSAT,
    attr: TypeAttr::empty(),
  };
  *ctx.cre.get_mut(dummy_me) = this;

  Success(dummy_me)
}
