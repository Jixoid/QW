use crate::front::patt_p::PattParser;
use crate::lexer::WK;
use crate::{ast::*, diagnostic::Message, front::{ParserContext, attr_p::AttrParser, expr_p::ExprParser, type_p::TypeParser}};


pub struct StmtParser {}

impl StmtParser {

  pub fn read_stmt<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<StmtId,  Message> {
    let attrs = AttrParser::read_attr(ctx)?;

    let l = ctx.lex.get()?;
    let id = match l.str() {
      "let"      => Self::read_let(ctx, AccessKind::IMM),
      "var"      => Self::read_let(ctx, AccessKind::MUT),
      "ret"      => Self::read_ret(ctx),
      "break"    => Self::read_break(ctx),
      "continue" => Self::read_continue(ctx),

      _ => {
        ctx.lex.store(l);
        let expr = ExprParser::read_expr(ctx, 0)?;

        let is_block_like = matches!(ctx.cre.get_expr(expr), Expr::If{..} | Expr::Block{..} | Expr::Match{..} | Expr::Loop{..} | Expr::While{..} | Expr::ForIn{..});
        if is_block_like {
          let end = ctx.lex.lex();
          if end.kind != WK::Semicolon {
            ctx.lex.store(end);
          }
        } else {
          ctx.lex.get()?.expect_equal_str(";")?;
        }

        let st = Stmt{vari: StmtVari::Expr(ExprStmt{expr})};
        
        Ok(ctx.cre.new_stmt(st))
      }
    }?;

    AttrParser::attach_attr(ctx, id, attrs);
    Ok(id)
  }


  pub fn read_let<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, acck: AccessKind) -> Result<StmtId, Message> {
    let item = PattParser::read_patt(ctx)?;

    let t1 = ctx.lex.get()?;
    let ty = if t1.kind == WK::Colon {
      Some(TypeParser::read_type(ctx, true)?)
    } else {
      ctx.lex.store(t1);
      None
    };

    let mut init = None;
    let t2 = ctx.lex.get()?;

    if t2.kind == WK::Assign {
      init = Some(ExprParser::read_expr(ctx, 0)?);
      ctx.lex.get()?.expect_equal_str(";")?;
    }
    else if t2.kind == WK::Semicolon { /* no init */ }
    else {
      t2.expect_equal_str2("=", ";")?;
    }

    let this = StmtVari::Let(LetStmt{
      item,
      kind: ty,
      init,
      acck
    });

    let st = Stmt{vari: this};

    Ok(ctx.cre.new_stmt(st))
  }

  pub fn read_ret<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<StmtId, Message> {
    let mut _c = ctx.lex.get()?;

    let label = if _c.kind == WK::Backtick {
      let a = Some(ctx.lex.get()?.expect_word()?);
      _c = ctx.lex.get()?;
      a
    } else { None };

    let val = if _c.kind != WK::Semicolon {
      ctx.lex.store(_c);
      let a = Some(ExprParser::read_expr(ctx, 0)?);
      _c = ctx.lex.get()?;
      a
    } else { None };

    _c.expect_equal_str(";")?;
    
  
    let this = StmtVari::Ret(RetStmt{
      label, val
    });

    let st = Stmt{vari: this};

    Ok(ctx.cre.new_stmt(st))
  }
  
  pub fn read_break<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<StmtId, Message> {
    let mut _c = ctx.lex.get()?;

    let label = if _c.kind == WK::Backtick {
      let a = Some(ctx.lex.get()?.expect_word()?);
      _c = ctx.lex.get()?;
      a
    } else { None };

    let val = if _c.kind != WK::Semicolon {
      ctx.lex.store(_c);
      let a = Some(ExprParser::read_expr(ctx, 0)?);
      _c = ctx.lex.get()?;
      a
    } else { None };
    
    _c.expect_equal_str(";")?;
    
  
    let this = StmtVari::Break(BreakStmt{
      label, val
    });

    let st = Stmt{vari: this};

    Ok(ctx.cre.new_stmt(st))
  }

  pub fn read_continue<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<StmtId, Message> {
    let mut _c = ctx.lex.get()?;

    let label = if _c.kind == WK::Backtick {
      let a = Some(ctx.lex.get()?.expect_word()?);
      _c = ctx.lex.get()?;
      a
    } else { None };

    _c.expect_equal_str(";")?;

  
    let this = StmtVari::Continue(ContinueStmt{
      label
    });

    let st = Stmt{vari: this};

    Ok(ctx.cre.new_stmt(st))
  }

}
