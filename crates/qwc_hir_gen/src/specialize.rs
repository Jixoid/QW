/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use itertools::Itertools;
use qwc_hir::{self as hir, Layout, PushOkApi, TypeAttr};

use crate::Ctx;


macro_rules! convert {
  ($ctx:ident, $rng:ident, $i:expr, $o:expr $(,)?) => {{
    let vec = $ctx.cre.extra_get($rng).map($i).collect_vec();
    
    let vec = vec.iter().map($o).collect_vec();
    
    $ctx.cre.extra(&vec)    
  }};
}


pub struct Specialization<'a> {
  pub args: &'a [hir::TypeId],
}

impl<'a> Specialization<'a> {

  pub fn spec_type(&self, ctx: &mut Ctx, id: hir::TypeId) -> hir::TypeId {
    let it = *ctx.get(id);

    match it.kind {
      hir::TypeKind::GenericType{idx} => return self.args[idx],

      hir::TypeKind::Unit
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

    let id = match it.kind {
      hir::TypeKind::Struct(rng) => {
        let rng = convert!(ctx, rng,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) { (name, kind) } else { panic!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.spec_type(ctx, kind)).push(ctx.cre)
          },
        );

        hir::Type {
          kind: hir::TypeKind::Struct(rng),
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
        }.push(ctx.cre)
      }

      hir::TypeKind::Tuple(rng) => {
        let rng = convert!(ctx, rng, |id| id, |&id| self.spec_type(ctx, id));

        hir::Type {
          kind: hir::TypeKind::Tuple(rng),
          layout: hir::Layout::new_static(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
        }.push(ctx.cre)
      }

      hir::TypeKind::Ref(kind, ism) => {
        let kind = self.spec_type(ctx, kind);

        ctx.tin.ty_ref(ctx.cre, kind, ism)
      }

      hir::TypeKind::Fun{self_kind, args, ret} => {
        let args = convert!(ctx, args,
          |id| {
            if let hir::Thing::NamedType(name, kind) = *ctx.cre.get(id) { (name, kind) } else { panic!() }
          },
          |&(name, kind)| {
            hir::Thing::NamedType(name, self.spec_type(ctx, kind)).push(ctx.cre)
          },
        );

        hir::Type {
          kind: hir::TypeKind::Fun { self_kind, args, ret: self.spec_type(ctx, ret) },
          layout: Layout::new_dsat(qwc_hir::LayoutBy::QW),
          attr: TypeAttr::empty(),
        }.push(ctx.cre)
      }

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_generic_type.insert(key, id);
    id
  }

  pub fn spec_expr(&self, ctx: &mut Ctx, id: hir::ExprId) -> hir::ExprId {
    let key = (id, self.args.to_vec());
    if let Some(&cached) = ctx.cmap.cache_generic_expr.get(&key) { return cached }

    let it = *ctx.get(id);
    
    let id = match it.kind {
      hir::ExprKind::GlobalRef(item) => {
        hir::Expr {
          kind: hir::ExprKind::GlobalRef(self.spec_item(ctx, item)),
          category: hir::ExprCategory::RValue,
          ety: self.spec_type(ctx, it.ety),
        }.push(ctx.cre)
      }
      
      hir::ExprKind::Block{stmt, expr} => {
        let stmt = convert!(ctx, stmt, |id| id, |&id| self.spec_expr(ctx, id));

        let expr = expr.map(|id| self.spec_expr(ctx, id));

        hir::Expr {
          kind: hir::ExprKind::Block {stmt, expr},
          category: hir::ExprCategory::RValue,
          ety: self.spec_type(ctx, it.ety),
        }.push(ctx.cre)
      }

      _ => todo!("{it:#?}")
    };

    ctx.cmap.cache_generic_expr.insert(key, id);
    id
  }

  pub fn spec_item(&self, ctx: &mut Ctx, id: hir::ItemId) -> hir::ItemId {
    let key = (id, self.args.to_vec());
    if let Some(&cached) = ctx.cmap.cache_generic_item.get(&key) { return cached }

    let it = *ctx.get(id);
    
    let id = match it.kind {
      hir::ItemKind::Function{kind, expr} => {
        let kind = self.spec_type(ctx, kind);
        let expr = self.spec_expr(ctx, expr);

        let mut path = it.path;
        for &arg in self.args {
          let sid = ctx.sin.get(&ctx.type_name(arg)).unwrap_or_else(|| ctx.sin.sid_entry());
          let arg_path = ctx.cre.push(hir::DefPath::Root(sid));
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
