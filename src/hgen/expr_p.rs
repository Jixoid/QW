use rustc_apfloat::{Float, ieee};

use crate::{ast::{self, AstKind}, diagnostic::Message, hgen::{GenContext, TypeGen}, hir::{self, AccessKind, BinaryOp, Expr, ExprVari, Type, UnaryOp, Value}, lexer::{SrcLoc, SrcSId}};


pub struct ExprGen;

impl ExprGen {

  pub fn low(ctx: &mut GenContext, id: ast::ExprId) -> Result<hir::ExprId, Message> {
    match ctx.ast.get_expr(id) {
      ast::Expr::Nick(span) => Self::low_nick(ctx, span),

      ast::Expr::Bool(span) => Self::low_bool(ctx, span),
      ast::Expr::Number(span) => Self::low_number(ctx, span),
      ast::Expr::String(span) => Self::low_string(ctx, span),

      ast::Expr::Binary { op, lhs, rhs } => Self::low_binary(ctx, *op, *lhs, *rhs),
      ast::Expr::Unary { op, val } => Self::low_unary(ctx, *op, *val),

      ast::Expr::Block { rng, expr, .. } => Self::low_block(ctx, rng, expr),
      
      ast::Expr::If { cond, then, elsb } => Self::low_if(ctx, *cond, *then, elsb),
      ast::Expr::While { cond, blok, .. } => Self::low_while(ctx, *cond, *blok),
      ast::Expr::Loop { blok, .. } => Self::low_loop(ctx, *blok),

      ast::Expr::Return { val, .. } => Self::low_return(ctx, val),
      ast::Expr::Break { val, .. } => Self::low_break(ctx, val),
      ast::Expr::Continue { .. } => Self::low_continue(ctx),

      ast::Expr::Let { item, kind, init, acck } => Self::low_let(ctx, *item, kind, init, *acck),
      ast::Expr::Call { callee, args } => Self::low_call(ctx, *callee, args),
      ast::Expr::Member(rng) => Self::low_member(ctx, rng),
      ast::Expr::Specialize { callee, args } => Self::low_specialize(ctx, *callee, args),

      //ast::Expr::Try( sub ) => ,

      ast::Expr::Unsafe( id ) => Self::low(ctx, *id),
      ast::Expr::Relaxed( id ) => Self::low(ctx, *id),

      _ => todo!(),
    }
  }


  fn low_nick(ctx: &mut GenContext, span: &crate::lexer::Span) -> Result<hir::ExprId, Message> {
    // Local scopes
    if let Some((ty, local_idx)) = ctx.local.lookup(span.sid()) {
      let ex = Expr { ty, vari: ExprVari::Local(local_idx) };
      return Ok(ctx.hir.new_expr(ex));
    }

    // Scope Lookup
    if let Some(target_id) = ctx.global.lookup(ctx, span.sid()) {
      match target_id.kind {
        AstKind::Decl => {
          let ty = ctx.ast.get_decl(ast::DeclId::new_from(target_id)).kind();
          
          let hir_ty = TypeGen::low(ctx, ty)?;
          let ex = Expr { ty: hir_ty, vari: ExprVari::Local(span.sid) };
          
          Ok(ctx.hir.new_expr(ex))
        }

        _ => {
          let unit_ty = ctx.hir.new_type(Type::Unit);
          let ex = Expr { ty: unit_ty, vari: ExprVari::Local(span.sid) };
          eprintln!("IGNORED unknown kind");
          Ok(ctx.hir.new_expr(ex))
        }
      }
    } else {
      Err(Message::error(*span, "unknown variable or identifier: `{}`", vec![ span.string(ctx.far) ]))
    }
  }


  fn low_bool(ctx: &mut GenContext, span: &crate::lexer::Span) -> Result<hir::ExprId, Message> {
    let ty = ctx.hir.new_type(Type::Bool);
    let ex = Expr { ty, vari: ExprVari::Lit(Value::Bool(span.str(ctx.far) == "true")) };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_number(ctx: &mut GenContext, span: &crate::lexer::Span) -> Result<hir::ExprId, Message> {
    let text = span.str(ctx.far);

    if text.contains('.') {
      
      let val = match text.parse::<ieee::Double>() {
        Ok(v) => v,
        Err(..) => { return Err(Message::error(*span, "number cannot converted float: `{}`", vec![ text.to_string() ])) }
      };

      let ty = ctx.hir.new_type(Type::Float(64));

      let ex = Expr { ty, vari: ExprVari::Lit(Value::Float(val.to_bits() as u64)) };
      
      Ok(ctx.hir.new_expr(ex))
    } else {
      let val = match text.parse::<i64>() {
        Ok(v) => v,
        Err(..) => { return Err(Message::error(*span, "number cannot converted integer: `{}`", vec![ text.to_string() ])) }
      };

      let ty = ctx.hir.new_type(Type::Int(32, true));
      
      let ex = Expr { ty, vari: ExprVari::Lit(Value::Int(val)) };
      
      Ok(ctx.hir.new_expr(ex))
    }
  }

  fn low_string(ctx: &mut GenContext, span: &crate::lexer::Span) -> Result<hir::ExprId, Message> {
    let ty = ctx.hir.new_type(Type::Str);
    let ex = Expr { ty, vari: ExprVari::Lit(Value::Str(span.sid())) };
    Ok(ctx.hir.new_expr(ex))
  }


  fn low_member(ctx: &mut GenContext, rng: &ast::Rng) -> Result<hir::ExprId, Message> {
    let parts = ctx.ast.get_extra(rng);
    if parts.len() < 2 {
      let unit_ty = ctx.hir.new_type(Type::Unit);
      let ex = Expr { ty: unit_ty, vari: ExprVari::Lit(Value::Null) };
      eprintln!("IGNORED unknown kind");
      return Ok(ctx.hir.new_expr(ex));
    }

    let base = Self::low(ctx, ast::ExprId::new_from(parts[0]))?;
    let base_ty = ctx.hir.get_expr(base).ty;

    let mut real_ty = base_ty;
    loop {
      let t = ctx.hir.get_type(real_ty);
      match t {
        Type::Ref(sub, _) | Type::Ptr(sub, _) => real_ty = *sub,
        _ => break,
      }
    }

    let field_ast = ctx.ast.get_expr(ast::ExprId::new_from(parts[1]));
    let field_span = if let ast::Expr::Nick(span) = field_ast {
      Some(*span)
    } else {
      None
    };

    let field_name = if let Some(span) = field_span {
      span.str(ctx.far)
    } else {
      ""
    };

    let target_t = ctx.hir.get_type(real_ty);
    match target_t {
      Type::Struct(rng) | Type::Trait(rng) | Type::Iface(rng) => {
        for (idx, id) in ctx.hir.get_extra(rng).iter().enumerate() {
          let thing = ctx.hir.get_thing(hir::ThingId::new_from(*id));
          if let hir::Thing::NamedType(name, field_ty) = thing {
            if name.str(ctx.far) == field_name {
              let ex = Expr {
                ty: *field_ty,
                vari: ExprVari::Member { base, index: idx as u32 },
              };
              return Ok(ctx.hir.new_expr(ex));
            }
          }
        }
      }

      _ => {}
    }

    // Generic Bound Trait Methods (requires T: Trait)
    for trait_tys in ctx.type_bounds.values() {
      for &tr_ty in trait_tys {
        let tr = ctx.hir.get_type(tr_ty);
        if let Type::Trait(funs_rng) | Type::Iface(funs_rng) = tr {
          for (idx, id) in ctx.hir.get_extra(funs_rng).iter().enumerate() {
            let thing = ctx.hir.get_thing(hir::ThingId::new_from(*id));
            if let hir::Thing::NamedType(name, method_ty) = thing {
              if name.str(ctx.far) == field_name {
                let ex = Expr {
                  ty: *method_ty,
                  vari: ExprVari::Member { base, index: idx as u32 },
                };
                return Ok(ctx.hir.new_expr(ex));
              }
            }
          }
        }
      }
    }

    // Target Type Method Lookup (from real_ty's AST scope)
    let mut ast_candidates = vec![];
    if let Some(&target_ast_ty) = ctx.type_to_ast.get(&real_ty) {
      let mut to_process = vec![target_ast_ty];
      while let Some(ast_id) = to_process.pop() {
        ast_candidates.push(ast_id.to_any());
        let ty_data = ctx.ast.get_type(ast_id);
        match ty_data {
          ast::Type::Specialize { base, .. } => {
            to_process.push(*base);
          }
          ast::Type::Nick(pos) => {
            if let Some(target_any) = ctx.global.lookup(ctx, pos.sid()) {
              ast_candidates.push(target_any);
              if target_any.kind == AstKind::Decl {
                let decl = ctx.ast.get_decl(ast::DeclId::new_from(target_any));
                if let ast::DeclVari::Using { kind } = &decl.vari {
                  to_process.push(*kind);
                }
              } else if target_any.kind == AstKind::Item {
                let item = ctx.ast.get_item(ast::ItemId::new_from(target_any));
                if let ast::ItemVari::Generic { ctn, .. } = &item.vari {
                  for &child in ctx.ast.get_extra(ctn) {
                    ast_candidates.push(child);
                    if child.kind == AstKind::Decl {
                      let decl = ctx.ast.get_decl(ast::DeclId::new_from(child));
                      if let ast::DeclVari::Using { kind } = &decl.vari {
                        to_process.push(*kind);
                      }
                    }
                  }
                }
              }
            }
          }
          _ => {}
        }
      }
    }

    for cand_any in ast_candidates {
      if let Some(scp) = ctx.ast.get_scope(cand_any) {
        for (k, v) in &scp.map {
          if ctx.ast.get_sid(*k) == field_name {
            if v.kind == AstKind::Decl {
              let decl = ctx.ast.get_decl(ast::DeclId::new_from(*v));
              if let ast::DeclVari::Fun { kind, .. } = &decl.vari {
                let method_ty = TypeGen::low(ctx, *kind)?;
                let ex = Expr {
                  ty: method_ty,
                  vari: ExprVari::Member { base, index: 0 },
                };
                return Ok(ctx.hir.new_expr(ex));
              }
            }
          }
        }
      }
    }

    // Method Lookup
    if let Some(self_ast_ty) = ctx.self_type {
      if let Some(scp) = ctx.ast.get_scope(self_ast_ty.to_any()) {
        for (k, v) in &scp.map {
          if ctx.ast.get_sid(*k) == field_name {
            if v.kind == AstKind::Decl {
              let decl = ctx.ast.get_decl(ast::DeclId::new_from(*v));
              if let ast::DeclVari::Fun { kind, .. } = &decl.vari {
                let method_ty = TypeGen::low(ctx, *kind)?;
                let ex = Expr {
                  ty: method_ty,
                  vari: ExprVari::Member { base, index: 0 },
                };
                return Ok(ctx.hir.new_expr(ex));
              }
            }
          }
        }
      }
    }

    if let Some(target) = ctx.global.lookup_name(ctx, field_name) {
      if target.kind == AstKind::Decl {
        let decl = ctx.ast.get_decl(ast::DeclId::new_from(target));
        if let ast::DeclVari::Fun { kind, .. } = &decl.vari {
          let method_ty = TypeGen::low(ctx, *kind)?;
          let ex = Expr {
            ty: method_ty,
            vari: ExprVari::Member { base, index: 0 },
          };
          return Ok(ctx.hir.new_expr(ex));
        }
      }
    }

    if let Some(span) = field_span {
      return Err(Message::error(span, "no field or method `{}` found on type", vec![ field_name.to_string() ]));
    }

    let unit_ty = ctx.hir.new_type(Type::Unit);
    let ex = Expr { ty: unit_ty, vari: ExprVari::Member { base, index: 0 } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_binary(ctx: &mut GenContext, op: ast::BinaryOp, lhs_id: ast::ExprId, rhs_id: ast::ExprId) -> Result<hir::ExprId, Message> {
    let lhs = Self::low(ctx, lhs_id)?;
    let rhs = Self::low(ctx, rhs_id)?;

    let lhs_ty = ctx.hir.get_expr(lhs).ty;
    let rhs_ty = ctx.hir.get_expr(rhs).ty;

    let h_op = match op {
      ast::BinaryOp::Add => BinaryOp::Add,
      ast::BinaryOp::Sub => BinaryOp::Sub,
      ast::BinaryOp::Mul => BinaryOp::Mul,
      ast::BinaryOp::Div => BinaryOp::Div,
      ast::BinaryOp::Rem => BinaryOp::Rem,

      ast::BinaryOp::Eq  => BinaryOp::Eq,
      ast::BinaryOp::Ne  => BinaryOp::Ne,
      ast::BinaryOp::Lt  => BinaryOp::Lt,
      ast::BinaryOp::Gt  => BinaryOp::Gt,
      ast::BinaryOp::Lte => BinaryOp::Lte,
      ast::BinaryOp::Gte => BinaryOp::Gte,
      
      ast::BinaryOp::And => BinaryOp::And,
      ast::BinaryOp::Or  => BinaryOp::Or,
      ast::BinaryOp::Xor => BinaryOp::Xor,
      ast::BinaryOp::Shl => BinaryOp::Shl,
      ast::BinaryOp::Shr => BinaryOp::Shr,
    };

    let res_ty = match h_op {
      BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Lte | BinaryOp::Gte |
      BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
        ctx.hir.new_type(Type::Bool)
      }

      _ => lhs_ty,
    };

    let ex = Expr { ty: res_ty, vari: ExprVari::Binary { op: h_op, lhs, rhs } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_unary(ctx: &mut GenContext, op: ast::UnaryOp, val_id: ast::ExprId) -> Result<hir::ExprId, Message> {
    let val = Self::low(ctx, val_id)?;
    let val_ty = ctx.hir.get_expr(val).ty;

    let (h_op, res_ty) = match op {
      ast::UnaryOp::Neg => (UnaryOp::Neg, val_ty),
      ast::UnaryOp::Poz => (UnaryOp::Neg, val_ty),
      ast::UnaryOp::Not => (UnaryOp::Not, ctx.hir.new_type(Type::Bool)),
      ast::UnaryOp::Ref => (UnaryOp::Ref(AccessKind::IMM), ctx.hir.new_type(Type::Ref(val_ty, AccessKind::IMM))),
      ast::UnaryOp::Addr => (UnaryOp::Ptr(AccessKind::IMM), ctx.hir.new_type(Type::Ptr(val_ty, AccessKind::IMM))),
      ast::UnaryOp::Deref => {
        let sub_ty = match ctx.hir.get_type(val_ty) {
          Type::Ref(sub, _) | Type::Ptr(sub, _) => *sub,
          _ => val_ty,
        };
        (UnaryOp::Deref, sub_ty)
      }
    };

    let ex = Expr { ty: res_ty, vari: ExprVari::Unary { op: h_op, val } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_block(ctx: &mut GenContext, rng: &ast::Rng, expr: &Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let mut stmts = vec![];

    for id in ctx.ast.get_extra(rng) {
      if id.kind == AstKind::Expr {
        let h_expr = Self::low(ctx, ast::ExprId::new_from(*id))?;
        stmts.push(h_expr);
      }
    }

    let final_expr = if let Some(e) = expr {
      Some(Self::low(ctx, *e)?)
    } else {
      None
    };

    let res_ty = if let Some(e) = final_expr {
      ctx.hir.get_expr(e).ty
    } else {
      ctx.hir.new_type(Type::Unit)
    };

    let stmts_rng = ctx.hir.new_extra_from(&stmts);
    let ex = Expr { ty: res_ty, vari: ExprVari::Block { stmts: stmts_rng, expr: final_expr } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_if(ctx: &mut GenContext, cond_id: ast::ExprId, then_id: ast::ExprId, elsb_id: &Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let cond = Self::low(ctx, cond_id)?;
    let then_b = Self::low(ctx, then_id)?;
    let then_ty = ctx.hir.get_expr(then_b).ty;

    let else_b = if let Some(e) = elsb_id {
      Some(Self::low(ctx, *e)?)
    } else {
      None
    };

    let ex = Expr { ty: then_ty, vari: ExprVari::If { cond, then_b, else_b } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_while(ctx: &mut GenContext, cond_id: ast::ExprId, blok_id: ast::ExprId) -> Result<hir::ExprId, Message> {
    let cond = Self::low(ctx, cond_id)?;
    let body = Self::low(ctx, blok_id)?;
    let unit_ty = ctx.hir.new_type(Type::Unit);

    let ex = Expr { ty: unit_ty, vari: ExprVari::While { cond, body } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_loop(ctx: &mut GenContext, blok_id: ast::ExprId) -> Result<hir::ExprId, Message> {
    let body = Self::low(ctx, blok_id)?;
    let unit_ty = ctx.hir.new_type(Type::Unit);

    let ex = Expr { ty: unit_ty, vari: ExprVari::Loop { body } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_return(ctx: &mut GenContext, val_id: &Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let val = if let Some(v) = val_id {
      Some(Self::low(ctx, *v)?)
    } else {
      None
    };

    let unit_ty = ctx.hir.new_type(Type::Unit);
    let ex = Expr { ty: unit_ty, vari: ExprVari::Ret { val } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_break(ctx: &mut GenContext, val_id: &Option<ast::ExprId>) -> Result<hir::ExprId, Message> {
    let val = if let Some(v) = val_id {
      Some(Self::low(ctx, *v)?)
    } else {
      None
    };

    let unit_ty = ctx.hir.new_type(Type::Unit);
    let ex = Expr { ty: unit_ty, vari: ExprVari::Break { val } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_continue(ctx: &mut GenContext) -> Result<hir::ExprId, Message> {
    let unit_ty = ctx.hir.new_type(Type::Unit);
    let ex = Expr { ty: unit_ty, vari: ExprVari::Continue };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_let(ctx: &mut GenContext, item: ast::PattId, kind: &Option<ast::TypeId>, init: &Option<ast::ExprId>, acck: ast::AccessKind) -> Result<hir::ExprId, Message> {
    let h_acck = match acck {
      ast::AccessKind::IMM => AccessKind::IMM,
      ast::AccessKind::MUT => AccessKind::MUT,
    };

    let init_expr = if let Some(i) = init {
      Some(Self::low(ctx, *i)?)
    } else {
      None
    };

    let ty = if let Some(k) = kind {
      TypeGen::low(ctx, *k)?
    } else if let Some(i) = init_expr {
      ctx.hir.get_expr(i).ty
    } else {
      ctx.hir.new_type(Type::Unit)
    };

    let patt = ctx.ast.get_patt(item);
    let local_idx = match patt {
      ast::Patt::One(span) => ctx.local.bind(span.sid(), ty),
      _ => 0,
    };

    let unit_ty = ctx.hir.new_type(Type::Unit);
    let ex = Expr {
      ty: unit_ty,
      vari: ExprVari::Let { local: local_idx, ty, init: init_expr, acck: h_acck },
    };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_call(ctx: &mut GenContext, callee_id: ast::ExprId, args_rng: &ast::Rng) -> Result<hir::ExprId, Message> {
    let callee = Self::low(ctx, callee_id)?;
    let callee_ty = ctx.hir.get_expr(callee).ty;

    let mut args = vec![];
    for arg_id in ctx.ast.get_extra(args_rng) {
      if arg_id.kind == AstKind::Expr {
        args.push(Self::low(ctx, ast::ExprId::new_from(*arg_id))?);
      }
    }

    let ret_ty = match ctx.hir.get_type(callee_ty) {
      Type::Fun { ret, .. } => ret.unwrap_or_else(|| ctx.hir.new_type(Type::Unit)),
      _ => ctx.hir.new_type(Type::Unit),
    };

    let h_args_rng = ctx.hir.new_extra_from(&args);
    let ex = Expr { ty: ret_ty, vari: ExprVari::Call { callee, args: h_args_rng } };
    Ok(ctx.hir.new_expr(ex))
  }

  fn low_specialize(ctx: &mut GenContext, callee_id: ast::ExprId, args_rng: &ast::Rng) -> Result<hir::ExprId, Message> {
    let callee_ast = ctx.ast.get_expr(callee_id);
    if let ast::Expr::Nick(span) = callee_ast {
      if span.str(ctx.far) == "cast" {
        for arg_id in ctx.ast.get_extra(args_rng) {
          if arg_id.kind == AstKind::Type {
            let target_ty = TypeGen::low(ctx, ast::TypeId::new_from(*arg_id))?;
            let fn_ty = ctx.hir.new_type(Type::Fun { args: 0..0, ret: Some(target_ty) });
            let ex = Expr { ty: fn_ty, vari: ExprVari::Lit(Value::Null) };
            return Ok(ctx.hir.new_expr(ex));
          }
        }
      }
    }

    Self::low(ctx, callee_id)
  }


  fn low_try(ctx: &mut GenContext, sub: ast::ExprId) -> Result<hir::ExprId, Message> {
    let ex = ExprGen::low(ctx, sub)?;
    let ty = ctx.hir.get_expr(ex).ty;

    let can = matches!(ctx.hir.get_type(ty), Type::Result(..));

    if can {
      Ok(ex)
    } else {
      panic!();
      //Err(Message::error(ctx.ast.get_expr(sub)., "expr is not result", vec![]))
    }
  }

}
