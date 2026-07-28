use crate::{ast::*, control::identy::IdentyId, diagnostic::*, front::{ParserContext, meta_p::MetaParser, stmt_p::StmtParser}};


pub struct ExprParser {}

impl ExprParser {

  fn get_prec(k: crate::lexer::WordKind) -> u8 {
    use crate::lexer::WordKind::*;
    match k {
      Assign | AssignmentAdd | AssignmentSub | AssignmentMul | AssignmentDiv |
      AssignmentRem | AssignmentBitwiseAnd | AssignmentBitwiseOr | AssignmentBitwiseXor |
      AssignmentLeftShift | AssignmentRighShift | AssignmentLogicalAnd | AssignmentLogicalOr | AssignmentLogicalXor => 1,
      LogicalOr => 2,
      LogicalAnd => 3,
      BitwiseOr => 4,
      BitwiseXor => 5,
      BitwiseAnd => 6,
      Equal | NotEqual => 7,
      AngleBeg | AngleEnd | SmallerEqual | BiggerEqual => 8,
      ShiftLeft | ShiftRigh => 9,
      Add | Sub => 10,
      Mul | Div | Rem => 11,
      _ => 0,
    }
  }
  

  pub fn read_expr<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let id = Self::read_expr_prec(ctx, 0)?;
    crate::front::attr_p::AttrParser::attach_attributes(ctx, id, attrs);
    Ok(id)
  }

  fn read_expr_prec<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, min_prec: u8) -> Result<IdentyId, Message<'a>> {
    let mut lhs = Self::read_primary(ctx)?;

    loop {
      let t = ctx.lex.lex();

      let prec = Self::get_prec(t.kind);
      if prec == 0 || prec < min_prec {
        ctx.lex.store(t);
        break;
      }

      let next_prec = if prec == 1 { prec } else { prec + 1 };
      let rhs = Self::read_expr_prec(ctx, next_prec)?;

      let expr = ExprVari::Binary(BinaryExpr {
        lhs,
        op: t.kind,
        rhs
      });
      lhs = ctx.mol.new_expr(Expr { vari: expr, ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0) });
    }

    Ok(lhs)
  }

  pub fn read_primary<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    let t = ctx.lex.get()?;

    match t.str() {
      "{" | "`" => {
        ctx.lex.store(t);
        Self::read_block(ctx)
      }
      "(" => {
        let val = Self::read_expr_prec(ctx, 0)?;
        MetaParser::expect_equal(ctx, ")")?;
        Ok(val)
      }
      _ => {
        if matches!(t.kind, crate::lexer::WordKind::At | crate::lexer::WordKind::Add | crate::lexer::WordKind::Sub | crate::lexer::WordKind::Bang | crate::lexer::WordKind::Tilde) {
          let val = Self::read_expr_prec(ctx, 12)?;
          let expr = ExprVari::Unary(UnaryExpr { op: t.kind, val });
          return Ok(ctx.mol.new_expr(Expr { vari: expr, ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0) }));
        }

        if t.str() == "if" {
          return Self::read_if(ctx);
        }
        if t.str() == "match" {
          return Self::read_match(ctx);
        }

        if t.kind == crate::lexer::WordKind::Word {
          let expr = ExprVari::Nick(NickExpr::new(ctx.mol, t));
          return Ok(ctx.mol.new_expr(Expr { vari: expr, ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0) }));
        }

        if t.kind == crate::lexer::WordKind::Number {
          let expr = ExprVari::Number(NumberExpr{pos: t});
          return Ok(ctx.mol.new_expr(Expr { vari: expr, ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0) }));
        }

        return Err(Message::error(t, String::from("expected expression, found `{}`"), vec![t.string()]));
      }
    }
  }


  pub fn read_block<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    let mut label = None;
    let mut t = ctx.lex.get()?;

    if t.kind == crate::lexer::WordKind::Backtick {
      let lbl = ctx.lex.get()?;
      if lbl.kind != crate::lexer::WordKind::Word {
        return Err(Message::error(lbl, String::from("expected identifier after backtick"), vec![]));
      }
      MetaParser::expect_equal(ctx, ":")?;
      label = Some(lbl);
      t = ctx.lex.get()?;
    }

    MetaParser::expect_equal_w(t, "{")?;

    let mut ctn = Vec::new();

    loop {
      let t = ctx.lex.get()?;
      if t.str() == "}" {
        break;
      }
      ctx.lex.store(t);

      let stmt = StmtParser::read_stmt(ctx)?;
      ctn.push(stmt);
    }

    let this = ExprVari::Block(BlockExpr{
      label,
      ctn
    });


    Ok(ctx.mol.new_expr(Expr { vari: this, ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0) }))
  }

  pub fn read_if<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    let cond = Self::read_expr(ctx)?;
    
    let t_block = ctx.lex.get()?;
    ctx.lex.store(t_block);
    let then_block = Self::read_block(ctx)?;

    let mut else_block = None;
    let t = ctx.lex.lex();
    if t.str() == "else" {
      let t_else = ctx.lex.get()?;
      ctx.lex.store(t_else);
      else_block = Some(Self::read_block(ctx)?);
    }
    else if t.str() == "ef" {
      else_block = Some(Self::read_if(ctx)?);
    }
    else {
      ctx.lex.store(t);
    }

    Ok(ctx.mol.new_expr(Expr { ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0), vari: ExprVari::If(IfExpr{cond, then_block, else_block }) }))
  }

  pub fn read_match<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId, Message<'a>> {
    let val = Self::read_expr(ctx)?;
    
    MetaParser::expect_equal(ctx, "{")?;
    let mut arms = Vec::new();

    loop {
      let t = ctx.lex.get()?;
      if t.str() == "}" {
        break;
      } else {
        ctx.lex.store(t);
      }
      
      let pat = Self::read_expr(ctx)?;
      MetaParser::expect_equal(ctx, "=>")?;
      let body = Self::read_expr(ctx)?;
      arms.push(MatchArm{ pat, body });

      let comma = ctx.lex.get()?;
      if comma.str() == "}" {
        break;
      }
      if comma.str() != "," {
        ctx.lex.store(comma);
      }
    }

    Ok(ctx.mol.new_expr(Expr { ty: crate::control::IdentyId::new(crate::control::IdentyKind::Null, 0, 0), vari: ExprVari::Match(MatchExpr { val, arms }) }))
  }

}
