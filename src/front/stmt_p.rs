use crate::{ast::*, control::IdentyId, diagnostic::Message, front::{ParserContext, expr_p::ExprParser, meta_p::MetaParser, type_p::TypeParser}};


pub struct StmtParser {}

impl StmtParser {

  pub fn read_stmt<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<IdentyId,  Message<'a>> {
    let attrs = crate::front::attr_p::AttrParser::read_attributes(ctx);
    let l = ctx.lex.get()?;
    let d = match l.str() {
      "let"    => Self::read_let(ctx, AccessKind::IMM),
      "var"    => Self::read_let(ctx, AccessKind::MUT),
      "ret"    => Self::read_ret(ctx),
      _ => {
        ctx.lex.store(l);
        let expr = ExprParser::read_expr(ctx)?;

        let is_block_like = matches!(ctx.mol.get_expr(expr).vari, ExprVari::If(_) | ExprVari::Block(_) | ExprVari::Match(_));
        if is_block_like {
          let end = ctx.lex.lex();
          if end.str() != ";" {
            ctx.lex.store(end);
          }
        } else {
          MetaParser::expect_equal(ctx, ";")?;
        }

        Ok(Stmt{vari: StmtVari::Expr(ExprStmt{expr})})
      }
    }?;

    let id = ctx.mol.new_stmt(d);
    crate::front::attr_p::AttrParser::attach_attributes(ctx, id, attrs);
    Ok(id)
  }


  pub fn read_let<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, acck: AccessKind) -> Result<Stmt<'a>, Message<'a>> {
    let name = ctx.lex.get()?;

    let t1 = ctx.lex.get()?;
    let mut ty = IdentyId::new(crate::control::IdentyKind::Null, 0, 0);

    if t1.str() == ":" {
      ty = TypeParser::read_type(ctx, true)?;
    } else {
      ctx.lex.store(t1);
    }

    let mut init = None;
    let t2 = ctx.lex.get()?;

    if t2.str() == "=" {
      init = Some(ExprParser::read_expr(ctx)?);
      MetaParser::expect_equal(ctx, ";")?;
    }
    else if t2.str() == ";" { /* no init */ }
    else {
      MetaParser::expect_equal2_w(t2, "=", ";")?;
    }

    let this = StmtVari::Let(LetStmt{
      name,
      kind: ty,
      init,
      acck
    });

    Ok(Stmt{vari: this})
  }

  pub fn read_ret<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Stmt<'a>, Message<'a>> {
    let exp = ExprParser::read_expr(ctx)?;
    MetaParser::expect_equal(ctx, ";")?;

    let this = StmtVari::Ret(RetStmt{
      val: exp
    });

    Ok(Stmt{vari: this})
  }

}
