use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Ident};
use qwc_hir::{self as hir, Const};
use qwc_resolve::{self as resolve, Resolver};

use crate::{Ctx, TypeLow, ctx, item_p::ItemLow};


pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
    if let Some(&id) = ctx.cmap.cache_expr.get(&id) { return Ok(id) }

    let it: &ast::Expr = ctx.src.get(id);
    
    let it = match it.kind {
      // Resolve
      ast::ExprKind::Nick(ident) => Self::low_nick(ctx, ident)?,
      ast::ExprKind::Path(rng)     => Self::low_path(ctx, rng)?,

      // ZST
      ast::ExprKind::Unit => Self::low_unit(ctx)?,

      // Primitive
      ast::ExprKind::Bool(.., v)  => Self::low_bool(ctx, v)?,
      ast::ExprKind::Number(span) => Self::low_number(ctx, span)?,

      // Block
      ast::ExprKind::Block{rng, expr, ..} => Self::low_block(ctx, rng, expr)?,
      
      // Assign
      ast::ExprKind::Assign{lhs, rhs} => Self::low_assign(ctx, lhs, rhs)?,

      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_expr.insert(id, it);

    Ok(it)
  }


  fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::ExprId, Message> {
    let (kind, lscp, _span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp).lookup(ident)?.get_k();
    Self::low_resolved(ctx, kind, lscp)
  }

  fn low_path(ctx: &mut Ctx, rng: ast::Rng) -> Result<hir::ExprId, Message> {
    let mut segment = vec![];

    for id in ctx.src.extra_get(rng) {
      let it: &ast::Expr = ctx.src.get(ast::ExprId::new_from(id));

      if let ast::ExprKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
    }

    let (kind, lscp, _span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp).resolve_path(&segment)?.get_k();
    Self::low_resolved(ctx, kind, lscp)
  }

  fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope) -> Result<hir::ExprId, Message> {
    let expr = match kind {
      resolve::ScopeKind::ExprParam(thing) => {
        if let ast::Thing::NamedType(_, ty) = ctx.src.get(thing) as &ast::Thing {
          let ty = TypeLow::low(ctx!(lscp -> ctx), *ty)?;

          // Post
          let this = hir::Expr{
            kind: hir::ExprKind::GenericExpr,
            ety: ty,
          };

          ctx.cre.push(this)
        }
        else { panic!() }
      }

      resolve::ScopeKind::Expr(id) => {
        let id = ItemLow::low(ctx!(lscp -> ctx), id)?.unwrap();
        let item: &hir::Item = ctx.cre.get(id);

        let kind = {
          let this = hir::Type::Ref(item.symbol_kind().unwrap());
          ctx.cre.push(this)
        };

        // Post
        let this = hir::Expr{
          kind: hir::ExprKind::GlobalRef(id),
          ety: kind,
        };

        ctx.cre.push(this)
      }

      resolve::ScopeKind::Type(ty) => {
        let span = (ctx.src.get(ty) as &ast::Type).pos;
        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
      }

      resolve::ScopeKind::Module(id) => {
        let span = (ctx.src.get(id) as &ast::Item).pos;
        return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
      }

      k @ _ => panic!("{k:#?}")
    };

    Ok(expr)
  }
  

  fn low_unit(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Const(Const::Unit),
      ety: ctx.cre.ty_unit()
    };

    Ok(ctx.cre.push(this))
  }


  fn low_bool(ctx: &mut Ctx, val: bool) -> Result<hir::ExprId, Message> {
    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Const(Const::Bool(val)),
      ety: ctx.cre.ty_bool()
    };

    Ok(ctx.cre.push(this))
  }

  fn low_number(ctx: &mut Ctx, span: Span) -> Result<hir::ExprId, Message> {
    let str = span.str(ctx.far);

    let val = match str.parse::<i32>() {
      Ok(v) => v,
      Err(..) => return Err(Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span))),
    };

    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Const(Const::Int(val)),
      ety: ctx.cre.ty_i32()
    };

    Ok(ctx.cre.push(this))
  }
  

  fn low_block(ctx: &mut Ctx, rng: ast::Rng, expr: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let stmt = {
      let mut stmt = vec![];

      for id in ctx.src.extra_get(rng) {
        let id = ast::ExprId::new_from(id);
        
        let id = Self::low(ctx, id)?;

        stmt.push(id);
      }

      ctx.cre.extra(&stmt)
    };

    let expr = match expr {
      None => None,
      Some(id) => Some(Self::low(ctx, id)?)
    };

    let ety = if let Some(expr) = expr { (ctx.cre.get(expr) as &hir::Expr).ety } else { ctx.cre.ty_unit() };


    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Block { stmt, expr },
      ety,
    };

    Ok(ctx.cre.push(this))
  }


  fn low_assign(ctx: &mut Ctx, lhs: ast::ExprId, rhs: ast::ExprId) -> Result<hir::ExprId, Message> {
    let lhs = ExprLow::low(ctx, lhs)?;
    let rhs = ExprLow::low(ctx, rhs)?;


    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Assign {lhs, rhs},
      ety: ctx.cre.ty_unit(),
    };
    
    Ok(ctx.cre.push(this))
  }

}
