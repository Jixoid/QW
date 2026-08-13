use crate::ast::AccessKind;
use crate::front::type_p::TypeParser;
use crate::lexer::WK;
use crate::{ast::{BinaryOp, Expr, ExprId, MatchArm, NumberConst, NumberExpr, UnaryOp}, diagnostic::*, front::{ParserContext, patt_p::PattParser}, lexer::WordKind};


pub struct ExprParser {}

impl ExprParser {

  fn get_infix_bp(op: &str) -> Option<(u8, u8)> {
    match op {
      "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<=" | ">>=" => Some((10, 11)),
      
      ".." => Some((15, 15)),
      
      "||" => Some((20, 21)),
      "^^" => Some((22, 23)),
      "&&" => Some((24, 25)),
      
      "==" | "!=" | "<" | "<=" | ">" | ">=" => Some((30, 31)),
      
      "|" => Some((40, 41)),
      "^" => Some((42, 43)),
      "&" => Some((44, 45)),
      
      "<<" | ">>" => Some((50, 51)),
      
      "+" | "-" => Some((60, 61)),
      
      "*" | "/" | "%" => Some((70, 71)),
      
      _ => None,
    }
  }

  fn parse_unary_op(op: &str) -> UnaryOp {
    match op {
      "-" => UnaryOp::Neg,
      "+" => UnaryOp::Poz,
      "!" => UnaryOp::Not,
      "*" => UnaryOp::Deref,
      "&" => UnaryOp::Ref,
      "^" => UnaryOp::BitNot,
      _ => unreachable!("Unknown unary operator: {}", op),
    }
  }

  fn parse_binary_op(op: &str) -> BinaryOp {
    match op {
      "+" => BinaryOp::Add,
      "-" => BinaryOp::Sub,
      "*" => BinaryOp::Mul,
      "/" => BinaryOp::Div,
      "%" => BinaryOp::Mod,
      "==" => BinaryOp::Eq,
      "!=" => BinaryOp::Neq,
      "<" => BinaryOp::Lt,
      ">" => BinaryOp::Gt,
      "<=" => BinaryOp::Lte,
      ">=" => BinaryOp::Gte,
      "&&" => BinaryOp::And,
      "||" => BinaryOp::Or,
      "=" => BinaryOp::Assign,
      _ => unreachable!("Unknown binary operator: {}", op),
    }
  }


  fn read_arg_list<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Vec<ExprId>, Message<'a>> {
    let mut args = Vec::new();
    
    let first = ctx.lex.get()?;
    if first.kind == WK::ParenEnd {
      return Ok(args);
    } else {
      ctx.lex.store(first);
    }

    loop {
      args.push(Self::read_expr(ctx, 0)?);

      let separator = ctx.lex.get()?;
      match separator.str() {
        "," => {
          let next = ctx.lex.get()?;
          if next.kind == WK::ParenEnd {
            break;
          }
          
          ctx.lex.store(next);
        }

        ")" => { break; }

        _ => return Err(Message::error(separator, format!("expected `,` or `)` in argument list, found `{}`", separator.str()), vec![])),
      }
    }

    Ok(args)
  }

  fn read_atom<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> { loop {
    let tok = ctx.lex.get()?;
    
    if tok.str().chars().next().map_or(false, |c| c.is_ascii_digit()) {
      if let Ok(num) = tok.str().parse::<i64>() {
        return Ok(ctx.cre.new_expr(Expr::Number(NumberExpr{pos: tok, num: NumberConst::I64(num)}) ));
      }
    }

    if tok.str().starts_with('"') {
      return Ok(ctx.cre.new_expr(Expr::String(tok)));
    }

    tok.expect_word()?;
    let ident = ctx.cre.get_str(tok.str());
    return Ok(ctx.cre.new_expr(Expr::Nick{pos: tok, idx: ident}));
  }}


  pub fn read_expr<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, min_bp: u8) -> Result<ExprId, Message<'a>> {
    let tok = ctx.lex.get()?;

    let mut lhs = match tok.str() {
      "-" | "+" | "!" | "*" | "&" | "^" => {
        let r_bp = 85;
        let val = Self::read_expr(ctx, r_bp)?;
        
        let op = Self::parse_unary_op(tok.str()); 
        
        ctx.cre.new_expr(Expr::Unary{op, val})
      }

      "(" => {
        let expr = Self::read_expr(ctx, 0)?;
        
        let _c = ctx.lex.get()?;

        if _c.kind == WordKind::ParenEnd { // (X)
          expr
        }
        else if _c.kind == WordKind::Comma { // (X, ...)
          let mut vals = vec![ expr ];
          let mut is_tuple = false;
          
          loop {
            let _c = ctx.lex.get()?;
            if _c.kind == WK::ParenEnd { break; }
            else { ctx.lex.store(_c); }

            let sub = Self::read_expr(ctx, 0)?;
            vals.push(sub);

            let _c = ctx.lex.get()?;
            if _c.kind == WK::ParenEnd { break; }
            else if _c.kind == WK::Comma { is_tuple = true; continue; }
            else {
              _c.expect_equal_str2(",", ")")?;
            }
          }

          if is_tuple || vals.len() != 1 {
            let ex = Expr::Tuple(vals);
            ctx.cre.new_expr(ex)
          } else {
            vals[0]
          }
        }
        else {
          _c.expect_equal_str2(")", ",")?;
          panic!()
        }
      }

      "{" | "`" => {
        ctx.lex.store(tok);
        Self::read_block(ctx)?
      }

      "if"    => Self::read_if(ctx)?,
      "match" => Self::read_match(ctx)?,
      "while" => Self::read_while(ctx)?,
      "loop"  => Self::read_loop(ctx)?,
      "for"   => Self::read_for(ctx)?,

      "let"      => Self::read_let(ctx, AccessKind::IMM)?,
      "var"      => Self::read_let(ctx, AccessKind::MUT)?,
      "ret"      => Self::read_ret(ctx)?,
      "break"    => Self::read_break(ctx)?,
      "continue" => Self::read_continue(ctx)?,

      _ => {
        ctx.lex.store(tok);
        Self::read_atom(ctx)? 
      }
    };

    
    loop {
      let next = ctx.lex.get()?;
      
      match next.str() {
        "." => {
          let field_tok = ctx.lex.get()?.expect_word()?;
          let field_idx = ctx.cre.get_str(field_tok.str());

          let sub = Expr::Nick{pos: field_tok, idx: field_idx};
          let sub_id = ctx.cre.new_expr(sub);

          let lookahead = ctx.lex.get()?;
          if lookahead.kind == WK::Scope {
            return Err(Message::error(lookahead, "cannot use `::` on a field access expression; use type name instead".to_string(), vec![]));
          }
          ctx.lex.store(lookahead);

          lhs = ctx.cre.new_expr(Expr::Member(vec![lhs, sub_id]));
        }

        "::" => {
          let field_tok = ctx.lex.get()?.expect_word()?;
          let field_idx = ctx.cre.get_str(field_tok.str());

          let sub = Expr::Nick{pos: field_tok, idx: field_idx};
          let sub_id = ctx.cre.new_expr(sub);

          lhs = ctx.cre.new_expr(Expr::Path(vec![lhs, sub_id]));
        }

        "(" => {
          let args = Self::read_arg_list(ctx)?;
          
          lhs = ctx.cre.new_expr(Expr::Call{callee: lhs, args});
        }

        "[" => {
          let mut args = Vec::new();
          
          loop {
            let sub = Self::read_expr(ctx, 0)?;
            args.push(sub);
            
            let sep = ctx.lex.get()?;
            match sep.str() {
              "]" => break,
              "," => {
                let next = ctx.lex.get()?;
                if next.kind == WK::SquareBracketEnd {
                  break;
                }
                ctx.lex.store(next);
              }
              _ => return Err(Message::error(sep, format!("expected `,` or `]` in index expression, found `{}`", sep.str()), vec![])),
            }
          }

          lhs = ctx.cre.new_expr(Expr::Index{callee: lhs, args});
        }

        "?" | "!" => {
          let this = match next.str() {
            "?" => Expr::Try(lhs),
            "!" => Expr::Unwrap(lhs),
            _ => panic!()
          };
          
          lhs = ctx.cre.new_expr(this);
        }

        _ => {
          ctx.lex.store(next);
          break;
        }
      }
    }

    
    loop {
      let op_tok = ctx.lex.get()?;
      
      let (_l_bp, r_bp) = match Self::get_infix_bp(op_tok.str()) {
        Some((l, r)) if l >= min_bp => (l, r),
        
        _ => { ctx.lex.store(op_tok); break; }
      };

      let rhs = Self::read_expr(ctx, r_bp)?;
      
      let op = Self::parse_binary_op(op_tok.str());

      lhs = ctx.cre.new_expr(Expr::Binary{op, lhs, rhs});
    }

    Ok(lhs)
  }


  pub fn read_block<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let mut label = None;
    let mut t = ctx.lex.get()?;

    if t.kind == crate::lexer::WordKind::Backtick {
      let lbl = ctx.lex.get()?;
      if lbl.kind != crate::lexer::WordKind::Word {
        return Err(Message::error(lbl, String::from("expected identifier after backtick"), vec![]));
      }
      ctx.lex.get()?.expect_equal_str(":")?;
      label = Some(lbl);
      t = ctx.lex.get()?;
    }

    t.expect_equal_str("{")?;

    let mut ctn = vec![];
    let mut expr = None;

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd {
        break;
      }
      ctx.lex.store(t);

      let ex_id = ExprParser::read_expr(ctx, 0)?;

      let end = ctx.lex.get()?;
      if end.kind == WK::Semicolon {
        ctn.push(ex_id.to_any());
      } else if end.kind == WK::CurlyBracketEnd {
        ctx.lex.store(end);
        expr = Some(ex_id);
      } else {
        let is_block_like = matches!(
          ctx.cre.get_expr(ex_id),
          Expr::If { .. } | Expr::Block { .. } | Expr::Match { .. } | Expr::Loop { .. } | Expr::While { .. } | Expr::ForIn { .. }
        );
        if is_block_like {
          ctx.lex.store(end);
          ctn.push(ex_id.to_any());
        } else {
          end.expect_equal_str(";")?;
          unreachable!()
        }
      }
    }


    let rng = ctx.cre.new_extra(ctn);

    let this = Expr::Block{
      label,
      rng,
      expr,
    };

    Ok(ctx.cre.new_expr(this))
  }


  pub fn read_if<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let cond = Self::read_expr(ctx, 0)?;
    
    let then = Self::read_block(ctx)?;

    let t: crate::lexer::Word<'_> = ctx.lex.lex();

    let elsb = if t.kind == WK::Else {
      let t_else = ctx.lex.get()?;
      ctx.lex.store(t_else);
      Some(Self::read_block(ctx)?)
    }
    else if t.kind == WK::Ef {
      Some(Self::read_if(ctx)?)
    }
    else {
      ctx.lex.store(t);
      None
    };


    let this = Expr::If{cond, then, elsb};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_match<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let cond = Self::read_expr(ctx, 0)?;
    
    ctx.lex.get()?.expect_equal_str("{")?;
    let mut arms = Vec::new();

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::CurlyBracketEnd {
        break;
      } else {
        ctx.lex.store(t);
      }
      
      let pat = Self::read_expr(ctx, 0)?;
      ctx.lex.get()?.expect_equal_str("=>")?;
      let body = Self::read_expr(ctx, 0)?;
      arms.push(MatchArm{ pat, body });

      let comma = ctx.lex.get()?;
      if comma.kind == WK::CurlyBracketEnd {
        break;
      }
      if comma.kind != WK::Comma {
        ctx.lex.store(comma);
      }
    }


    let this = Expr::Match{cond, arms};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_loop<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let blok = Self::read_block(ctx)?;

    let _c = ctx.lex.get()?;
    let elsb = if _c.kind == WK::Else {
      Some(Self::read_expr(ctx, 0)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let this = Expr::Loop{blok, elsb};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_while<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let cond = Self::read_expr(ctx, 0)?;
    let blok = Self::read_block(ctx)?;

    let _c = ctx.lex.get()?;
    let elsb = if _c.kind == WK::Else {
      Some(Self::read_expr(ctx, 0)?)
    } else {
      ctx.lex.store(_c);
      None
    };


    let this = Expr::While{blok, cond, elsb};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_for<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let vars = PattParser::read_patt(ctx)?;

    ctx.lex.get()?.expect_equal_str("in")?;
    
    let iter = Self::read_expr(ctx, 0)?;

    let blok = Self::read_block(ctx)?;

    let _c = ctx.lex.get()?;
    let elsb = if _c.kind == WK::Else {
      Some(Self::read_expr(ctx, 0)?)
    } else {
      ctx.lex.store(_c);
      None
    };


    let this = Expr::ForIn{ vars, iter, blok, elsb };

    Ok(ctx.cre.new_expr(this))
  }


  pub fn read_let<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, acck: AccessKind) -> Result<ExprId, Message<'a>> {
    let item = PattParser::read_patt(ctx)?;

    let _c = ctx.lex.get()?;
    let kind = if _c.kind == WK::Colon {
      Some(TypeParser::read_type(ctx, true)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    
    let _c = ctx.lex.get()?;
    let init = if _c.kind == WK::Assign {
      Some(ExprParser::read_expr(ctx, 0)?)
    }
    else {
      ctx.lex.store(_c);
      None
    };


    let this = Expr::Let{item, kind, init, acck};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_ret<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let _c = ctx.lex.get()?;
    let label = if _c.kind == WK::Backtick {
      Some(ctx.lex.get()?.expect_word()?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let _c = ctx.lex.get()?;
    let val = if _c.kind != WK::Semicolon {
      Some(ExprParser::read_expr(ctx, 0)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let this = Expr::Return{label, val};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_break<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let _c = ctx.lex.get()?;
    let label = if _c.kind == WK::Backtick {
      Some(ctx.lex.get()?.expect_word()?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let _c = ctx.lex.get()?;
    let val = if _c.kind != WK::Semicolon {
      Some(ExprParser::read_expr(ctx, 0)?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let this = Expr::Break{label, val};

    Ok(ctx.cre.new_expr(this))
  }

  pub fn read_continue<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<ExprId, Message<'a>> {
    let _c = ctx.lex.get()?;
    let label = if _c.kind == WK::Backtick {
      Some(ctx.lex.get()?.expect_word()?)
    } else {
      ctx.lex.store(_c);
      None
    };

    let this = Expr::Continue{label};

    Ok(ctx.cre.new_expr(this))
  }

}
