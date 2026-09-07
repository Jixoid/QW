use qwc_diagnostic::{Message, Span};
use qwc_ast::{self as ast, Ident};
use qwc_hir::{self as hir, Value};
use qwc_resolve::{self as resolve, Resolver};

use crate::{Ctx, TypeLow, ctx};


pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let it: &ast::Expr = src.get(id);
    
    let it = match it.kind {
      ast::ExprKind::Nick(ident) => Self::low_nick(ctx, ident)?,

      ast::ExprKind::Unit => Self::low_unit(ctx)?,

      ast::ExprKind::Bool(.., v)  => Self::low_bool(ctx, v)?,
      ast::ExprKind::Number(span) => Self::low_number(ctx, span)?,

      ast::ExprKind::Block{rng, expr, ..} => Self::low_block(ctx, rng, expr)?,
      
      _ => todo!("{:#?}", it)
    };

    Ok(it)
  }


  fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::ExprId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let ty = match Resolver::new(scp, far, lscp).lookup(ident)?.get_k() {

      (resolve::ScopeKind::ExprParam(thing), lscp, ..) => {
        match src.get(thing) as &ast::Thing {
          ast::Thing::NamedType(_, ty) => {
            let ty = TypeLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), *ty)?;
            
            let this = hir::Expr{
              kind: hir::ExprKind::GenericExpr,
              ety: ty,
            };

            cre.push(this)
          }

          c @_ => panic!("{c:#?}")
        }
      }

      (k @_, ..) => panic!("{k:#?}")
    };

    Ok(ty)
  }
  

  fn low_unit(ctx: &mut Ctx) -> Result<hir::ExprId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Lit(Value::Unit),
      ety: cre.ty_unit()
    };

    Ok(cre.push(this))
  }


  fn low_bool(ctx: &mut Ctx, val: bool) -> Result<hir::ExprId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Lit(Value::Bool(val)),
      ety: cre.ty_bool()
    };

    Ok(cre.push(this))
  }

  fn low_number(ctx: &mut Ctx, span: Span) -> Result<hir::ExprId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let str = span.str(far);

    let val = match str.parse::<i32>() {
      Ok(v) => v,
      Err(..) => return Err(Message::error(span, "cannot convert to int: `{}`", &[str])),
    };

    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Lit(Value::Int(val)),
      ety: cre.ty_i32()
    };

    Ok(cre.push(this))
  }
  

  fn low_block(ctx: &mut Ctx, rng: ast::Rng, expr: Option<ast::ExprId>) -> Result<hir::ExprId, Message> { ctx!(ctx => cre, sum, src, sin, far, scp, lscp);
    let stack = {
      let mut stack = vec![];

      for id in src.extra_get(rng) {
        let it: &ast::Expr = src.get(ast::ExprId::new_from(id));
        
        if let ast::ExprKind::Let{kind, ..} = it.kind {
          if kind.is_none() {
            return Err(Message::error(it.pos, "please specify the type", &[]))
          }

          let kind = kind.unwrap();
          let kind = TypeLow::low(ctx!(cre,sum,src,sin,far,scp,lscp), kind)?;

          stack.push(kind);
        }
      }

      cre.extra(&stack)
    };

    let expr = match expr {
      None => None,
      Some(id) => Some(Self::low(ctx!(cre,sum,src,sin,far,scp,lscp), id)?)
    };

    let ety = if let Some(expr) = expr { (cre.get(expr) as &hir::Expr).ety } else { cre.ty_unit() };


    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Block { stack, expr },
      ety,
    };

    Ok(cre.push(this))
  }

}
