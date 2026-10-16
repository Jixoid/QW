/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::{Itertools, izip};
use qwc_diagnostic::{Label, Message, msg::*};
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

  pub fn spec_type(ctx: &mut Ctx, expected: &[hir::ThingId], received: &[ast::AnyId], generic: hir::ItemId, kind: hir::TypeId) -> Result<hir::TypeId, Message> {
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
          let id = TypeLow::low(ctx, id)?;
          args.push(id.to_any());
        }
        
        (true, ast::id::NodeKind::Expr) => {
          let id = ast::ExprId::from_any(rec_id);
          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(ctx.src.get(id).pos)));
        }
        
        (false, ast::id::NodeKind::Type) => {
          let id = ast::TypeId::from_any(rec_id);
          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(ctx.src.get(id).pos)));
        }
        
        (false, ast::id::NodeKind::Expr) => {
          let hir::Thing::NamedType(_, expected_ty) = expected_thing else { unreachable!() };
          let id = ast::ExprId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          let fctx = FunCtx::new_const(pos);
          let expr = ExprLow::low(ctx, &fctx, id)?;
          let expr_ty = ctx.cre.get(expr).ety;
          TypeMatch::matches_pos(ctx, expected_ty, expr_ty, pos)?;
          args.push(expr.to_any());
        }
        
        _ => unreachable!(),
      }
    }

    let id = Specialization{generic, args: &args}.sub_spec_type(ctx, kind);

    Ok(id)
  }

  pub fn spec_expr(ctx: &mut Ctx, fctx: &FunCtx, expected: &[hir::ThingId], received: &[ast::AnyId], generic: hir::ItemId, expr: hir::ExprId) -> Result<hir::ExprId, Message> {
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
          let ty = TypeLow::low(ctx, id)?;
          args.push(ty.to_any());
        }

        (true, ast::id::NodeKind::Expr) => {
          let id = ast::ExprId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["type", "expr"]), Label::new_pos(pos)));
        }

        (false, ast::id::NodeKind::Type) => {
          let id = ast::TypeId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          return Err(Message::error(EXPECTED_BUT_FOUND.args(&["expr", "type"]), Label::new_pos(pos)));
        }

        (false, ast::id::NodeKind::Expr) => {
          let hir::Thing::NamedType(_, expected_ty) = expected_thing else { unreachable!() };
          let id = ast::ExprId::from_any(rec_id);
          let pos = ctx.src.get(id).pos;
          let expr = ExprLow::low(ctx, fctx, id)?;
          let expr_ty = ctx.cre.get(expr).ety;
          TypeMatch::matches_pos(ctx, expected_ty, expr_ty, pos)?;
          args.push(expr.to_any());
        }

        _ => unreachable!(),
      }
    }

    let id = Specialization{generic, args: &args}.sub_spec_expr(ctx, expr);
    
    Ok(id)
  }

  
  fn sub_spec_type(&self, ctx: &mut Ctx, id: hir::TypeId) -> hir::TypeId {
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

    let mut path = it.path;
    for &id in self.args {
      if id.kind() != hir::id::NodeKind::Type { continue }

      let id = hir::TypeId::from_any(id);
      let arg_path = ctx.get(id).path;
      path = ctx.cre.push(hir::DefPath::Spec { base: path, spec: arg_path });
    }

    let id = match it.kind {
      hir::TypeKind::Struct(rng) => {
        let rng = convert!(ctx, rng,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.get(id) { (name, kind) } else { unreachable!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.sub_spec_type(ctx, kind)).push(ctx.cre)
          },
        );

        hir::Type {
          kind: hir::TypeKind::Struct(rng),
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Tuple(rng) => {
        let rng = convert!(ctx, rng, |id| id, |&id| self.sub_spec_type(ctx, id));

        hir::Type {
          kind: hir::TypeKind::Tuple(rng),
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Ref(kind, ism) => {
        let kind = self.sub_spec_type(ctx, kind);

        ctx.tin.ty_ref(ctx.cre, ctx.ideps, kind, ism)
      }
      
      hir::TypeKind::Slice(kind) => {
        let kind = self.sub_spec_type(ctx, kind);
  
        ctx.tin.ty_slice(ctx.cre, ctx.ideps, kind)
      }
      
      hir::TypeKind::Array(kind, ext) => {
        let kind = self.sub_spec_type(ctx, kind);
        let ext = self.sub_spec_expr(ctx, ext);
  
        hir::Type {
          kind: hir::TypeKind::Array(kind, ext),
          layout: Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Fun{self_kind, args, ret} => {
        let args = convert!(ctx, args,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.get(id) { (name, kind) } else { unreachable!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.sub_spec_type(ctx, kind)).push(ctx.cre)
          },
        );

        hir::Type {
          kind: hir::TypeKind::Fun { self_kind, args, ret: self.sub_spec_type(ctx, ret) },
          layout: Layout::new_dsat(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre)
      }

      hir::TypeKind::AssociatedType { trait_ty, idx } => {
        let trait_ty = self.sub_spec_type(ctx, trait_ty);
        hir::Type {
          kind: hir::TypeKind::AssociatedType { trait_ty, idx },
          layout: it.layout,
          attr: it.attr,
          path: it.path,
        }.push(ctx.cre)
      }

      hir::TypeKind::Trait(methods, types) => {
        let new_trait_ty = hir::Type {
          kind: hir::TypeKind::Error,
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
          path,
        }.push(ctx.cre);

        ctx.cmap.cache_generic_type.insert(key.clone(), new_trait_ty);

        let methods = convert!(ctx, methods,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.get(id) { (name, kind) } else { unreachable!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.sub_spec_type(ctx, kind)).push(ctx.cre)
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
              Some(kind) => hir::Thing::NamedType(name, self.sub_spec_type(ctx, kind)).push(ctx.cre),
              None => hir::Thing::Name(name).push(ctx.cre),
            }
          },
        );

        let this = hir::Type {
          kind: hir::TypeKind::Trait(methods, types),
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
    id
  }

  fn sub_spec_expr(&self, ctx: &mut Ctx, id: hir::ExprId) -> hir::ExprId {
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
          kind: hir::ExprKind::GlobalRef(self.sub_spec_item(ctx, item)),
          category: hir::ExprCategory::RValue,
          ety: self.sub_spec_type(ctx, it.ety),
        }.push(ctx.cre)
      }
      
      hir::ExprKind::Block{stmt, expr} => {
        let stmt = convert!(ctx, stmt, |id| id, |&id| self.sub_spec_expr(ctx, id));

        let expr = expr.map(|id| self.sub_spec_expr(ctx, id));

        hir::Expr {
          kind: hir::ExprKind::Block {stmt, expr},
          category: hir::ExprCategory::RValue,
          ety: self.sub_spec_type(ctx, it.ety),
        }.push(ctx.cre)
      }
      hir::ExprKind::IntCast(expr) => {
        hir::Expr {
          kind: hir::ExprKind::IntCast(self.sub_spec_expr(ctx, expr)),
          category: hir::ExprCategory::RValue,
          ety: self.sub_spec_type(ctx, it.ety),
        }.push(ctx.cre)
      }

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_generic_expr.insert(key, id);
    id
  }

  fn sub_spec_item(&self, ctx: &mut Ctx, id: hir::ItemId) -> hir::ItemId {
    let key = (id, self.args.to_vec());
    if let Some(&cached) = ctx.cmap.cache_generic_item.get(&key) { return cached }

    let it = *ctx.get(id);
    
    let id = match it.kind {
      hir::ItemKind::Function{kind, expr} => {
        let kind = self.sub_spec_type(ctx, kind);
        let expr = self.sub_spec_expr(ctx, expr);

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
