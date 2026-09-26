use qwc_diagnostic::{Label, Message, Span, msg::*};
use qwc_ast::{self as ast, Ident};
use qwc_hir::{self as hir, Const, ExprCategory};
use qwc_resolve::{self as resolve, Resolver};

use crate::{Ctx, TypeLow, ctx, item_p::ItemLow};


pub struct ExprLow;

impl ExprLow {

  pub fn low(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
    if let Some(&id) = ctx.cmap.cache_expr.get(&id) { return Ok(id) }

    let it: &ast::Expr = ctx.src.get(id);
    
    use ast::ExprKind::*;

    let it = match it.kind {
      // Resolve
      Nick(ident) => Self::low_nick(ctx, ident)?,
      Path(rng)     => Self::low_path(ctx, rng)?,

      // ZST
      Unit => Self::low_unit(ctx)?,

      // Primitive
      Bool(.., v)  => Self::low_bool(ctx, v)?,
      Number(span) => Self::low_number(ctx, span)?,

      // Operator
      Unary{op, val} => Self::low_unary(ctx, op, val)?,

      // Variable
      Let{item, kind, init, ism} => Self::low_let(ctx, item, kind, init, ism, it.pos)?,

      // Block
      Block{rng, expr, ..} => Self::low_block(ctx, rng, expr)?,
      
      // Assign
      Assign{lhs, rhs, op_span} => Self::low_assign(ctx, lhs, rhs, op_span)?,

      // Loop
      Loop{blok, elsb} => Self::low_loop(ctx, it, blok, elsb)?,

      // Route
      Return{val, ..} => Self::low_return(ctx, val)?,
      Break{val, ..} => Self::low_break(ctx, val)?,
      Continue{..} => Self::low_continue(ctx)?,

      // Operator
      Binary{op, lhs, rhs} => Self::low_binary(ctx, op, lhs, rhs)?,

      // Branch
      If{cond, then, elsb} => Self::low_if(ctx, it, cond, then, elsb)?,

      _ => todo!("{:#?}", it)
    };

    ctx.cmap.cache_expr.insert(id, it);

    Ok(it)
  }


  // Resolve
  fn low_nick(ctx: &mut Ctx, ident: Ident) -> Result<hir::ExprId, Message> {
    let (kind, lscp, span) = Resolver::with_locals(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods, Some(ctx.loc)).lookup(ident)?.get_k();
    Self::low_resolved(ctx, kind, lscp, span)
  }

  fn low_path(ctx: &mut Ctx, rng: ast::ExprRng) -> Result<hir::ExprId, Message> {
    let mut segment = vec![];

    for id in ctx.src.extra_get(rng) {
      let it: &ast::Expr = ctx.src.get(id);

      if let ast::ExprKind::Nick(ident) = it.kind { segment.push(ident) } else { panic!() }
    }

    let (kind, lscp, span) = Resolver::new(ctx.scp, ctx.sin, ctx.lscp, ctx.ideps, ctx.imods).resolve_path(&segment)?.get_k();
    Self::low_resolved(ctx, kind, lscp, span)
  }

  fn low_resolved(ctx: &mut Ctx, kind: resolve::ScopeKind, lscp: &resolve::Scope, span: Span) -> Result<hir::ExprId, Message> {
    let expr = match kind {
      resolve::ScopeKind::Ast(ast_kind) => match ast_kind {
        resolve::ScopeKindAst::ExprParam(thing) => {
          if let ast::Thing::NamedType(_, ty) = ctx.src.get(thing) as &ast::Thing {
            let ty = TypeLow::low(ctx!(lscp -> ctx), *ty)?;

            // Post
            let this = hir::Expr{
              kind: hir::ExprKind::GenericExpr,
              category: ExprCategory::RValue,
              ety: ty,
            };

            ctx.cre.push(this)
          }
          else { panic!() }
        }

        resolve::ScopeKindAst::Expr(id) => {
          let id = ItemLow::low(ctx!(lscp -> ctx), id)?.unwrap();
          let it: &hir::Item = ctx.cre.get(id);

          let kind = it.symbol_kind().unwrap();
          
          let ism = it.assignable().unwrap();


          // Post
          let this = hir::Expr{
            kind: hir::ExprKind::GlobalRef(id),
            category: ExprCategory::lvalue(ism), // TODO!
            ety: kind,
          };

          ctx.cre.push(this)
        }

        resolve::ScopeKindAst::Type(..) | resolve::ScopeKindAst::TypeParam(..) => {
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
        }

        resolve::ScopeKindAst::Module(id) => {
          let span = (ctx.src.get(id) as &ast::Item).pos;
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
        }

        resolve::ScopeKindAst::Local(id) => {
          let local = ctx.loc.get_local(id);
          let this = hir::Expr {
            kind: hir::ExprKind::LocalRef(id),
            category: ExprCategory::lvalue(local.ism),
            ety: local.ty,
          };
          ctx.cre.push(this)
        }
      },

      resolve::ScopeKind::Hir(hir_kind) => match hir_kind {
        resolve::ScopeKindHir::Expr(id, ty) => {
          let ty = ctx.low_hir_type(ty);
          let kind = ctx.tin.ty_ref(ctx.cre, ty, false); // TODO!

          let this = hir::Expr {
            kind: hir::ExprKind::GlobalRef(id),
            category: ExprCategory::RValue,
            ety: kind,
          };

          ctx.cre.push(this)
        }

        resolve::ScopeKindHir::Type(..) | resolve::ScopeKindHir::Module(..) => {
          return Err(Message::error(EXPECTED_BUT_FOUND, Label::new_pos(span)));
        }
      },
    };

    Ok(expr)
  }
  

  // ZST
  fn low_unit(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Const(Const::Unit),
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_unit()
    };

    Ok(ctx.cre.push(this))
  }


  // Primitive
  fn low_bool(ctx: &mut Ctx, val: bool) -> Result<hir::ExprId, Message> {
    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Const(Const::Bool(val)),
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_bool()
    };

    Ok(ctx.cre.push(this))
  }

  fn low_number(ctx: &mut Ctx, span: Span) -> Result<hir::ExprId, Message> {
    let str = span.str(ctx.far);

    let val = str.parse::<i32>().map_err(|_| Message::error(CANNOT_CONVERT_TO_INT, Label::new_pos(span)))?;


    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Const(Const::Int(val)),
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_i32()
    };

    Ok(ctx.cre.push(this))
  }
  

  // Operator
  fn low_unary(ctx: &mut Ctx, op: ast::UnaryOp, val: ast::ExprId) -> Result<hir::ExprId, Message> {
    use ast::UnaryOp::*;
    
    match op {
      Deref => Self::low_unary_deref(ctx, val),

      _ => todo!("{op:#?}")
    }
  }

  fn low_unary_deref(ctx: &mut Ctx, id: ast::ExprId) -> Result<hir::ExprId, Message> {
    let id = ExprLow::low(ctx, id)?;
    let it: &hir::Expr = ctx.cre.get(id);
    let ty: &hir::Type = ctx.cre.get(it.ety);

    use hir::TypeKind::*;

    let (ty, ism) = match ty.kind {
      Ref(ty, ism) => (ty, ism),

      _ => todo!("{ty:#?}")
    };


    // Post
    let this = hir::Expr {
      kind: hir::ExprKind::Deref(id),
      category: ExprCategory::lvalue(ism),
      ety: ty,
    };

    Ok(ctx.cre.push(this))
  }


  // Variable
  fn low_let(ctx: &mut Ctx, item: ast::PattId, kind: Option<ast::TypeId>, init: Option<ast::ExprId>, ism: bool, pos: Span) -> Result<hir::ExprId, Message> {
    let patt: &ast::Patt = ctx.src.get(item);
    let (name, span) = match patt {
      ast::Patt::One(ident) => (ident.sid(), (*ident).into()),
      _ => todo!("patterns in let bindings not implemented yet"),
    };

    let init_id = match init {
      Some(id) => id,
      None => {
        let name_str = ctx.sin.str(name);
        return Err(Message::error(VARIABLE_REQUIRES_INITIALIZER
          .args(&[
            name_str
          ]),
          Label::new_pos(pos),
        ));
      }
    };

    let init_hir = ExprLow::low(ctx, init_id)?;
    let init_ty = (ctx.cre.get(init_hir) as &hir::Expr).ety;

    let var_ty = if let Some(kind_id) = kind {
      let declared_ty = TypeLow::low(ctx, kind_id)?;
      if init_ty != declared_ty {
        todo!("{:#?}, {:#?}",
          ctx.cre.get(declared_ty) as &hir::Type,
          ctx.cre.get(init_ty) as &hir::Type,
        );
      }
      declared_ty
    } else {
      init_ty
    };

    let local_id = ctx.loc.insert(name, var_ty, ism, span);

    let this = hir::Expr {
      kind: hir::ExprKind::Let {
        local: local_id,
        init: init_hir,
      },
      category: ExprCategory::lvalue(ism),
      ety: ctx.tin.ty_unit(),
    };

    Ok(ctx.cre.push(this))
  }


  // Block
  fn low_block(ctx: &mut Ctx, rng: ast::ExprRng, expr: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    ctx.loc.push_scope();

    let res = (|| {
      let mut stmt = vec![];

      for id in ctx.src.extra_get(rng) {
        let id = Self::low(ctx, id)?;
        stmt.push(id);
      }

      let stmt_rng = ctx.cre.extra(&stmt);
      let expr = expr.map(|id| ExprLow::low(ctx, id)).transpose()?;
      Ok((stmt_rng, expr))
    })();

    ctx.loc.pop_scope();
    let (stmt, expr) = res?;

    let ety = if let Some(expr) = expr { (ctx.cre.get(expr) as &hir::Expr).ety } else { ctx.tin.ty_unit() };

    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Block { stmt, expr },
      category: ExprCategory::RValue,
      ety,
    };

    Ok(ctx.cre.push(this))
  }


  // Assign
  fn low_assign(ctx: &mut Ctx, lhs: ast::ExprId, rhs: ast::ExprId, op_span: Span) -> Result<hir::ExprId, Message> {
    let lhs_hir = ExprLow::low(ctx, lhs)?;
    let rhs_hir = ExprLow::low(ctx, rhs)?;


    /* Check Assignable */ {
      let lhs_ast: &ast::Expr = ctx.src.get(lhs);
      let lhs_hir: &hir::Expr = ctx.cre.get(lhs_hir);

      if lhs_hir.category == ExprCategory::RValue {
        return Err(Message::error(CANNOT_ASSIGN_RVALUE, Label::new_pos(op_span))
          .add(Label::new(lhs_ast.pos, CANNOT_ASSIGN_TO_THIS_EXPRESSION))
        );
      }

      if lhs_hir.category == ExprCategory::LValueImm {
        return Err(Message::error(CANNOT_ASSIGN_IMMUTABLE, Label::new_pos(op_span))
          .add(Label::new(lhs_ast.pos, CANNOT_ASSIGN_TO_THIS_EXPRESSION))
        );
      }
    }


    let lhs_ty = (ctx.cre.get(lhs_hir) as &hir::Expr).ety;
    let rhs_ty = (ctx.cre.get(rhs_hir) as &hir::Expr).ety;

    if lhs_ty != rhs_ty {
      todo!("{:#?}, {:#?}",
        ctx.cre.get(lhs_ty) as &hir::Type,
        ctx.cre.get(rhs_ty) as &hir::Type,
      );
    }

    // Post
    let this = hir::Expr{
      kind: hir::ExprKind::Assign {lhs: lhs_hir, rhs: rhs_hir},
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_unit(),
    };
    
    Ok(ctx.cre.push(this))
  }


  // Loop
  fn low_loop(ctx: &mut Ctx, it: &ast::Expr, blok: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let blok = ExprLow::low(ctx, blok)?;

    let elsb = elsb.map(|id| ExprLow::low(ctx, id)).transpose()?;

    let ety = find_ety(ctx, it.pos, blok, elsb)?;


    // Post
    let this = hir::Expr {
      kind: hir::ExprKind::Loop {blok, elsb},
      category: ExprCategory::RValue,
      ety,
    };
    
    Ok(ctx.cre.push(this))
  }

  
  // Route
  fn low_return(ctx: &mut Ctx, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let val = val.map(|id| ExprLow::low(ctx, id)).transpose()?;


    // Post
    let item = hir::Expr {
      kind: hir::ExprKind::Return(val),
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_never(),
    };

    Ok(ctx.cre.push(item))
  }

  fn low_break(ctx: &mut Ctx, val: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let val = val.map(|id| ExprLow::low(ctx, id)).transpose()?;


    // Post
    let item = hir::Expr {
      kind: hir::ExprKind::Break(val),
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_never(),
    };

    Ok(ctx.cre.push(item))
  }

  fn low_continue(ctx: &mut Ctx) -> Result<hir::ExprId, Message> {
    // Post
    let item = hir::Expr {
      kind: hir::ExprKind::Continue,
      category: ExprCategory::RValue,
      ety: ctx.tin.ty_never(),
    };

    Ok(ctx.cre.push(item))
  }


  // Operator
  fn low_binary(_ctx: &mut Ctx, _op: ast::BinaryOp, _lhs: ast::ExprId, _rhs: ast::ExprId) -> Result<hir::ExprId, Message> {
    //let op_trait = match op {
    //  _ => todo!("{op:#?}")
    //};

    todo!()
  }


  // Branch
  fn low_if(ctx: &mut Ctx, it: &ast::Expr, cond: ast::ExprId, then: ast::ExprId, elsb: Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let cond_ast: &ast::Expr = ctx.src.get(cond);
    let cond = ExprLow::low(ctx, cond)?;
    let cond_ty = (ctx.cre.get(cond) as &hir::Expr).ety;

    if cond_ty != ctx.tin.ty_bool() {
      let found_ty = ctx.type_name(cond_ty);
      return Err(Message::error(
        EXPECTED_BUT_FOUND.args(&["bool", &found_ty]),
        Label::new(cond_ast.pos, EXPECTED_X.args(&["bool"])),
      ));
    }


    let then = ExprLow::low(ctx, then)?;

    let elsb = elsb.map(|id| ExprLow::low(ctx, id)).transpose()?;

    let ety = find_ety(ctx, it.pos, then, elsb)?;


    // Post
    let this = hir::Expr {
      kind: hir::ExprKind::If { cond, then, elsb },
      category: ExprCategory::RValue,
      ety,
    };

    Ok(ctx.cre.push(this))
  }

}


fn find_ety(ctx: &mut Ctx, pos: Span, blok: hir::ExprId, elsb: Option<hir::ExprId>) -> Result<hir::TypeId, Message> {
  let ty_blok = (ctx.cre.get(blok) as &hir::Expr).ety;
  let opt_ty_elsb = elsb.map(|id| (ctx.cre.get(id) as &hir::Expr).ety);

  let ty_unit = ctx.tin.ty_unit();
  let ty_never = ctx.tin.ty_never();

  let it = match opt_ty_elsb {
    None => ty_blok,
      
    Some(ty_elsb) => {
      if ty_blok == ty_never {
        // Durum 1: Blok '!' döndürüyor (örn. hep panic atıyor). Else'in tipini al.
        ty_elsb
      } else if ty_elsb == ty_never {
        // Durum 2: Else '!' döndürüyor. Bloğun tipini al.
        ty_blok
      } else if ty_blok == ty_elsb {
        // Durum 3: İkisi de aynı tip.
        ty_blok
      } else if ty_blok == ty_unit {
        // Durum 4: Blok hiçbir şey döndürmüyor, Else T döndürüyor -> ?T
        ctx.tin.ty_option(ctx.cre, ty_elsb)
      } else if ty_elsb == ty_unit {
        // Durum 5: Blok T döndürüyor, Else hiçbir şey döndürmüyor -> ?T
        ctx.tin.ty_option(ctx.cre, ty_blok)
      } else {
        // Durum 6: Birbiriyle alakasız iki tip (örn: i32 ve String)
        return Err(Message::error(LOOP_BRANCHES_HAVE_INCOMPATIBLE_TYPE, Label::new_pos(pos)));
      }
    }
  };

  Ok(it)
}
