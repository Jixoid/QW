/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, Layout, PushOkApi, TypeAttr};
use qwc_ast as ast;

use crate::{Ctx, ExprLow, FunCtx, TypeLow, type_p::TypeMatch};


macro_rules! convert {
  ($ctx:ident, $rng:ident, $i:expr, $o:expr $(,)?) => {{
    let vec = $ctx.extra_get($rng).map($i).collect_vec();
    
    let vec = vec.iter().map($o).collect_vec();
    
    $ctx.cre.extra(&vec)    
  }};
}


pub struct Specialization<'a> {
  generic: hir::ItemId,
  args: &'a [hir::AnyId],
}

impl<'a> Specialization<'a> {

  pub fn spec_type(ctx: &mut Ctx, sum: &mut Summary, expected: &[hir::ThingId], received: &[ast::AnyId], generic: hir::ItemId, kind: hir::TypeId) -> Status<hir::TypeId> {
    assert_eq!(expected.len(), received.len());

    let mut args = vec![];
      
    for (&exp_id, &rec_id) in izip!(expected, received) {
      let expected_thing = *ctx.get(exp_id);
      let is_expected_type = match expected_thing {
        hir::Thing::Name(_) => true,
        hir::Thing::NamedType(_, ty) => ty == ctx.prims.ty_type,
        _ => unreachable!(),
      };
      
      match (is_expected_type, rec_id.kind()) {
        (true, ast::id::NodeKind::Type) => {
          let id = ast::TypeId::from_any(rec_id);
          let id = TypeLow::low(ctx, sum, id)?;
          args.push(id.to_any());
        }
        
        (true, ast::id::NodeKind::Expr) => {
          let id = ast::ExprId::from_any(rec_id);
          ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(ctx.src.get(id).pos)));
        }
        
        (false, ast::id::NodeKind::Type) => {
          let id = ast::TypeId::from_any(rec_id);
          ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(ctx.src.get(id).pos)));
        }
        
        (false, ast::id::NodeKind::Expr) => {
          let hir::Thing::NamedType(_, expected_ty) = expected_thing else { unreachable!() };
          let id = ast::ExprId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          let fctx = FunCtx::new_const(pos);
          let expr = ExprLow::low(ctx, &fctx, sum, id)?;
          let expr_ty = ctx.cre.get(expr).ety;
          TypeMatch::matches_pos(ctx, sum, expected_ty, expr_ty, pos)?;
          args.push(expr.to_any());
        }
        
        _ => unreachable!(),
      }
    }

    let id = Specialization{generic, args: &args}.sub_spec_type(ctx, sum, kind);

    Status::Success(id)
  }

  pub fn spec_expr(ctx: &mut Ctx, sum: &mut Summary, fctx: &FunCtx, expected: &[hir::ThingId], received: &[ast::AnyId], generic: hir::ItemId, expr: hir::ExprId) -> Status<hir::ExprId> {
    assert_eq!(expected.len(), received.len());
    
    let mut args = vec![];

    for (&exp_id, &rec_id) in izip!(expected, received) {
      let expected_thing = *ctx.get(exp_id);
      let is_expected_type = match expected_thing {
        hir::Thing::Name(_) => true,
        hir::Thing::NamedType(_, ty) => ty == ctx.prims.ty_type,
        _ => unreachable!(),
      };

      match (is_expected_type, rec_id.kind()) {
        (true, ast::id::NodeKind::Type) => {
          let id = ast::TypeId::from_any(rec_id);
          let ty = TypeLow::low(ctx, sum, id)?;
          args.push(ty.to_any());
        }

        (true, ast::id::NodeKind::Expr) => {
          let id = ast::ExprId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(pos)));
        }

        (false, ast::id::NodeKind::Type) => {
          let id = ast::TypeId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          ret_err!(sum, Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(pos)));
        }

        (false, ast::id::NodeKind::Expr) => {
          let hir::Thing::NamedType(_, expected_ty) = expected_thing else { unreachable!() };
          let id = ast::ExprId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          let expr = ExprLow::low(ctx, fctx, sum, id)?;
          let expr_ty = ctx.cre.get(expr).ety;
          TypeMatch::matches_pos(ctx, sum, expected_ty, expr_ty, pos)?;
          args.push(expr.to_any());
        }

        _ => unreachable!(),
      }
    }

    let id = Specialization{generic, args: &args}.sub_spec_expr(ctx, sum, expr);
    
    Status::Success(id)
  }

  
  fn sub_spec_type(&self, ctx: &mut Ctx, sum: &mut Summary, id: hir::TypeId) -> hir::TypeId {
    let it = *ctx.get(id);

    match it.kind {
      hir::TypeKind::GenericType{generic, idx} => {
        if generic == self.generic {
          return hir::TypeId::from_any(self.args[idx]);
        } else {
          return id;
        }
      }

      hir::TypeKind::GenericSelfType
      | hir::TypeKind::Unit
      | hir::TypeKind::Never
      | hir::TypeKind::Bool
      | hir::TypeKind::Char
      | hir::TypeKind::Str
      | hir::TypeKind::Int(..)
      | hir::TypeKind::ArchInt(..)
      | hir::TypeKind::Float(..)
      | hir::TypeKind::Bit(..)
      | hir::TypeKind::Error => return id,

      _ => {},
    }
    
    let key = (id, self.args.to_vec());
    if let Some(&cached) = ctx.cmap.cache_generic_type.get(&key) {
      return cached;
    }

    if let Some((orig_generic, base_ty, orig_args)) = ctx.cmap.cache_generic_type_rev.get(&id).cloned() {
      let mut any_changed = false;
      let mut new_args = Vec::with_capacity(orig_args.len());
      for &arg in &orig_args {
        let new_arg = match arg.kind() {
          hir::id::NodeKind::Type => {
            let ty_id = hir::TypeId::from_any(arg);
            let spec_ty = self.sub_spec_type(ctx, sum, ty_id);
            if spec_ty != ty_id { any_changed = true; }
            spec_ty.to_any()
          }
          hir::id::NodeKind::Expr => {
            let expr_id = hir::ExprId::from_any(arg);
            let spec_expr = self.sub_spec_expr(ctx, sum, expr_id);
            if spec_expr != expr_id { any_changed = true; }
            spec_expr.to_any()
          }
          _ => arg,
        };
        new_args.push(new_arg);
      }

      if any_changed {
        let res = Specialization {
          generic: orig_generic,
          args: &new_args,
        }.sub_spec_type(ctx, sum, base_ty);
        ctx.cmap.cache_generic_type.insert(key, res);
        return res;
      } else {
        ctx.cmap.cache_generic_type.insert(key, id);
        return id;
      }
    }

    let mut path = it.path;
    for &id in self.args {
      if id.kind() != hir::id::NodeKind::Type { continue }

      let id = hir::TypeId::from_any(id);
      let arg_path = ctx.get(id).path;
      path = ctx.cre.push(hir::DefPath::Spec { base: path, spec: arg_path });
    }

    let it_id = id;
    let id = match it.kind {
      hir::TypeKind::Struct (rng) => {
        let rng = convert!(ctx, rng,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.get(id) { (name, kind) } else { unreachable!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.sub_spec_type(ctx, sum, kind)).push(ctx.cre)
          },
        );

        hir::Type {
          kind: hir::TypeKind::Struct(rng),
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Tuple (rng) => {
        let rng = convert!(ctx, rng, |id| id, |&id| self.sub_spec_type(ctx, sum, id));

        hir::Type {
          kind: hir::TypeKind::Tuple(rng),
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Enum(..) => id,

      hir::TypeKind::Ref (kind, ism) => {
        let kind = self.sub_spec_type(ctx, sum, kind);

        ctx.tin.ty_ref(ctx.cre, ctx.ideps, kind, ism)
      }

      hir::TypeKind::Option (kind) => {
        let kind = self.sub_spec_type(ctx, sum, kind);
  
        ctx.tin.ty_option(ctx.cre, ctx.ideps, kind)
      }

      hir::TypeKind::Slice (kind) => {
        let kind = self.sub_spec_type(ctx, sum, kind);
  
        ctx.tin.ty_slice(ctx.cre, ctx.ideps, kind)
      }

      hir::TypeKind::Array (kind, ext) => {
        let kind = self.sub_spec_type(ctx, sum, kind);
        let ext = self.sub_spec_expr(ctx, sum, ext);
  
        hir::Type {
          kind: hir::TypeKind::Array(kind, ext),
          layout: Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Fun {self_kind, args, ret} => {
        let args = convert!(ctx, args,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.get(id) { (name, kind) } else { unreachable!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.sub_spec_type(ctx, sum, kind)).push(ctx.cre)
          },
        );

        hir::Type {
          kind: hir::TypeKind::Fun { self_kind, args, ret: self.sub_spec_type(ctx, sum, ret) },
          layout: Layout::new_dsat(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::AssociatedType {trait_ty, idx} => {
        let trait_ty = self.sub_spec_type(ctx, sum, trait_ty);
        hir::Type {
          kind: hir::TypeKind::AssociatedType { trait_ty, idx },
          layout: it.layout,
          attr: it.attr,
          path: it.path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Trait {methods, types, reqs} => {
        let new_trait_ty = hir::Type {
          kind: hir::TypeKind::Error,
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre);

        ctx.cmap.cache_generic_type.insert(key.clone(), new_trait_ty);
        ctx.cmap.cache_generic_type_rev.insert(new_trait_ty, (self.generic, it_id, self.args.to_vec()));

        let methods = convert!(ctx, methods,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.get(id) { (name, kind) } else { unreachable!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.sub_spec_type(ctx, sum, kind)).push(ctx.cre)
          },
        );

        let types = convert!(ctx, types,
          |id| {
            match *ctx.get(id) {
              hir::Thing::NamedType(name, kind) => (name, Some(kind)),
              hir::Thing::Name(name) => (name, None),
              _ => unreachable!(),
            }
          },
          |&(name, kind)| {
            match kind {
              Some(kind) => hir::Thing::NamedType(name, self.sub_spec_type(ctx, sum, kind)).push(ctx.cre),
              None => hir::Thing::Name(name).push(ctx.cre),
            }
          },
        );

        let reqs = convert!(ctx, reqs, |id| id,
          |&id| self.sub_spec_type(ctx, sum, id),
        );

        let this = hir::Type {
          kind: hir::TypeKind::Trait{methods, types, reqs},
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        };
        *ctx.cre.get_mut(new_trait_ty) = this;

        new_trait_ty
      }

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_generic_type.insert(key, id);
    ctx.cmap.cache_generic_type_rev.insert(id, (self.generic, it_id, self.args.to_vec()));
    id
  }

  fn sub_spec_expr(&self, ctx: &mut Ctx, sum: &mut Summary, id: hir::ExprId) -> hir::ExprId {
    let key = (id, self.args.to_vec());
    if let Some(&cached) = ctx.cmap.cache_generic_expr.get(&key) { return cached }

    let it = *ctx.get(id);
    
    let id = match it.kind {
      hir::ExprKind::GenericExpr{generic, idx} => {
        if generic == self.generic {
          return hir::ExprId::from_any(self.args[idx]);
        } else {
          return id;
        }
      }

      hir::ExprKind::GlobalRef(item) => {
        hir::Expr {
          kind: hir::ExprKind::GlobalRef(self.sub_spec_item(ctx, sum, item)),
          category: hir::ExprCategory::RValue,
          ety: self.sub_spec_type(ctx, sum, it.ety),
        }.push(ctx.cre)
      }
      
      hir::ExprKind::Block{stmt, expr} => {
        let stmt = convert!(ctx, stmt, |id| id, |&id| self.sub_spec_expr(ctx, sum, id));

        let expr = expr.map(|id| self.sub_spec_expr(ctx, sum, id));

        hir::Expr {
          kind: hir::ExprKind::Block {stmt, expr},
          category: hir::ExprCategory::RValue,
          ety: self.sub_spec_type(ctx, sum, it.ety),
        }.push(ctx.cre)
      }
      hir::ExprKind::IntCast(expr) => {
        hir::Expr {
          kind: hir::ExprKind::IntCast(self.sub_spec_expr(ctx, sum, expr)),
          category: hir::ExprCategory::RValue,
          ety: self.sub_spec_type(ctx, sum, it.ety),
        }.push(ctx.cre)
      }

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_generic_expr.insert(key, id);
    id
  }

  fn sub_spec_item(&self, ctx: &mut Ctx, sum: &mut Summary, id: hir::ItemId) -> hir::ItemId {
    let key = (id, self.args.to_vec());
    if let Some(&cached) = ctx.cmap.cache_generic_item.get(&key) { return cached }

    let it = *ctx.get(id);
    
    let id = match it.kind {
      hir::ItemKind::Function{kind, expr} => {
        let kind = self.sub_spec_type(ctx, sum, kind);
        let expr = self.sub_spec_expr(ctx, sum, expr);

        let mut path = it.path;
        for &id in self.args {
          if id.kind() != hir::id::NodeKind::Type { continue }

          let id = hir::TypeId::from_any(id);
          let arg_path = ctx.get(id).path;
          path = ctx.cre.push(hir::DefPath::Spec { base: path, spec: arg_path });
        }

        hir::Item {
          kind: hir::ItemKind::Function { kind, expr },
          name: it.name,
          path,
          vis: it.vis,
          attr: it.attr,
        }.push(ctx.cre)
      }
      
      _ => todo!("{it:#?}")
    };
    
    ctx.cmap.cache_generic_item.insert(key, id);
    id
  }
  
}
