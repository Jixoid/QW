use qwc_ast::{BinaryOp, Expr, ExprId, ExprKind, IdentSave, Rng, Thing, UnaryOp};
use qwc_diagnostic::Message;
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx, PattParser, TypeParser, WordCheck};


pub struct ExprParser;

impl ExprParser {

  // Public
  pub fn read_expr(ctx: &mut Ctx) -> Result<ExprId, Message> {
    Self::read_expr_sub(ctx, 0)
  }

  fn read_expr_sub(ctx: &mut Ctx, min_bp: u8) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let mut lhs = match lex.peek()?.kind() {
      WK::CurlyBracketBeg | WK::Backtick => Self::pre_block(ctx!(cre, sin, far, lex, sum))?,

      WK::True | WK::False => Self::pre_bool(ctx!(cre, sin, far, lex, sum))?,
      WK::Number => Self::pre_number(ctx!(cre, sin, far, lex, sum))?,
      WK::String => Self::pre_string(ctx!(cre, sin, far, lex, sum))?,

      WK::ParenBeg => Self::pre_tuple(ctx!(cre, sin, far, lex, sum))?,
      WK::SquareBracketBeg => Self::pre_propagate(ctx!(cre, sin, far, lex, sum))?,

      WK::If    => Self::pre_if(ctx!(cre, sin, far, lex, sum))?,
      WK::Match => Self::pre_match(ctx!(cre, sin, far, lex, sum))?,
      WK::Loop  => Self::pre_loop(ctx!(cre, sin, far, lex, sum))?,
      WK::While => Self::pre_while(ctx!(cre, sin, far, lex, sum))?,
      WK::For   => Self::pre_for(ctx!(cre, sin, far, lex, sum))?,
      
      WK::Ret      => Self::pre_ret(ctx!(cre, sin, far, lex, sum))?,
      WK::Break    => Self::pre_break(ctx!(cre, sin, far, lex, sum))?,
      WK::Continue => Self::pre_continue(ctx!(cre, sin, far, lex, sum))?,
      
      WK::Unsafe  => Self::pre_unsafe(ctx!(cre, sin, far, lex, sum))?,
      WK::Relaxed => Self::pre_relaxed(ctx!(cre, sin, far, lex, sum))?,

      WK::SelfB => Self::pre_self_b(ctx!(cre, sin, far, lex, sum))?,
      WK::SelfS => Self::pre_self_s(ctx!(cre, sin, far, lex, sum))?,
      
      WK::Let | WK::Var => Self::pre_let(ctx!(cre, sin, far, lex, sum))?,
      
      WK::Sub | WK::Add | WK::Bang | WK::BitwiseAnd | WK::At => Self::pre_unary(ctx!(cre, sin, far, lex, sum))?,

      _ => Self::pre_nick(ctx!(cre, sin, far, lex, sum))?,
    };

    loop {
      lhs = match lex.peek()?.kind() {
        WK::Dot => Self::post_member_spec(ctx!(cre, sin, far, lex, sum), lhs)?,

        WK::Scope => Self::post_scope(ctx!(cre, sin, far, lex, sum), lhs)?,

        WK::ParenBeg         => Self::post_call(ctx!(cre, sin, far, lex, sum), lhs)?,
        WK::SquareBracketBeg => Self::post_index(ctx!(cre, sin, far, lex, sum), lhs)?,

        WK::Bang     => Self::post_unwrap(ctx!(cre, sin, far, lex, sum), lhs)?,
        WK::Question => Self::post_try(ctx!(cre, sin, far, lex, sum), lhs)?,

        _ => break
      }
    }

    loop {
      let op_tok = lex.peek()?;
      
      let (_, r_bp) = match Self::get_infix_bp(op_tok.kind()) {
        Some((l, r)) if l >= min_bp => { lex.bump()?; (l, r) },
        _ => break,
      };

      let rhs = Self::read_expr_sub(ctx!(cre, sin, far, lex, sum), r_bp)?;
      
      let kind = match Self::parse_binary_op(op_tok.kind()) {
        BinOp::Bin(op)      => ExprKind::Binary{op, lhs, rhs},
        BinOp::AssignOp(op) => ExprKind::AssignOp{op, lhs, rhs},

        BinOp::Assign   => ExprKind::Assign{lhs, rhs},
        BinOp::Exchange => ExprKind::Exchange{lhs, rhs},
      };

      let this = Expr{
        pos: lex.pos_extend(op_tok),
        kind 
      };

      lhs = cre.push(this);
    }

    Ok(lhs)
  }



  // Ex
  fn post_member_spec(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    lex.get()?;

    match lex.peek()?.kind() {
      WK::AngleBeg => Self::post_specialize(ctx!(cre, sin, far, lex, sum), lhs),
      _ => Self::post_member(ctx!(cre, sin, far, lex, sum), lhs),
    }
  }


  fn post_member(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;

    let rng = {
      let sub = Self::pre_nick(ctx!(cre, sin, far, lex, sum))?;
      
      let mut ctn = vec![lhs, sub];

      loop {
        match lex.peek_k()? {
          (WK::Dot, _) => {
            lex.bump()?;
            ctn.push(Self::pre_nick(ctx!(cre, sin, far, lex, sum))?);
          }

          (WK::Scope, c) => return Err(Message::error(c, "cannot use `::` on a field access expression; use type name instead", &[])),

          _ => break
        }
      }
      
      cre.extra(&ctn)
    };
      

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Member(rng)
    };

    Ok(cre.push(this))
  }

  fn post_scope(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let rng = {
      let sub = Self::pre_nick(ctx!(cre, sin, far, lex, sum))?;
      
      let mut ctn = vec![lhs, sub];

      loop {
        if lex.peek()?.kind() == WK::Scope {
          lex.bump()?;
          ctn.push(Self::pre_nick(ctx!(cre, sin, far, lex, sum))?);
        } else {
          break
        }
      }
      
      cre.extra(&ctn)
    };
      

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Path(rng)
    };

    Ok(cre.push(this))
  }


  fn post_call(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let args = if lex.peek()?.kind() == WK::ParenEnd {
      lex.bump()?;
      Rng::empty()
    } else {
      let mut args = vec![];

      loop {
        args.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::ParenEnd { lex.bump()?; break },
          
          (WK::ParenEnd, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::ParenEnd)?,
        }
      }

      cre.extra(&args)
    };

    
    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Call{ callee: lhs, args }
    };

    Ok(cre.push(this))
  }

  fn post_index(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let args = if lex.peek()?.kind() == WK::SquareBracketEnd {
      lex.bump()?;
      Rng::empty()
    } else {
      let mut args = vec![];

      loop {
        args.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::SquareBracketEnd { lex.bump()?; break },
          
          (WK::SquareBracketEnd, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::SquareBracketEnd)?,
        }
      }

      cre.extra(&args)
    };

    
    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Index{ callee: lhs, args }
    };

    Ok(cre.push(this))
  }


  fn post_try(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Try(lhs)
    };

    Ok(cre.push(this))
  }

  fn post_unwrap(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Unwrap(lhs)
    };

    Ok(cre.push(this))
  }


  fn post_specialize(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let args = if lex.peek()?.kind() == WK::AngleEnd {
      lex.bump()?;
      Rng::empty()
    } else {
      let mut args = vec![];

      loop {
        let arg = match lex.peek_k()? {
          (WK::String | WK::Number | WK::CurlyBracketBeg, _) => ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?.to_any(),
          
          _ => TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?.to_any(),
        };
        
        args.push(arg);

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::AngleEnd { lex.bump()?; break },
          
          (WK::AngleEnd, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::AngleEnd)?,
        }
      }

      cre.extra_any(&args)
    };

    
    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Spec{ callee: lhs, args }
    };

    Ok(cre.push(this))
  }

  

  // Sub
  pub fn pre_block(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let label = match start.kind() {
      WK::CurlyBracketBeg => None,

      WK::Backtick => {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (_, c) => return Err(Message::error(c, "expected identifier after `", &[lex.str(c)])),
        };
        
        lex.get()?.expect_kind(WK::Colon)?;
        lex.get()?.expect_kind(WK::CurlyBracketBeg)?;
        
        Some(name.into())
      }
      
      _ => start.panic_kind2(WK::Backtick, WK::CurlyBracketBeg)?
    };

    let (rng, expr) = {
      let mut ctn = vec![];
      let mut expr = None;
      
      loop {
        if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break }
        
        let ex_id = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
        match lex.peek_k()? {
          (WK::Semicolon, _) => { lex.bump()?; ctn.push(ex_id); },
          
          (WK::CurlyBracketEnd, _) => { expr = Some(ex_id); }
          
          (_, c) => {
            if cre.get::<Expr>(ex_id).is_like_blok() {
              ctn.push(ex_id);
            } else {
              c.panic_kind(WK::Semicolon)?;
            }
          }
        }
      }

      (cre.extra(&ctn), expr)
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Block{ label, rng, expr }
    };

    Ok(cre.push(this))
  }


  fn pre_self_b(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::SelfB()
    };

    Ok(cre.push(this))
  }

  fn pre_self_s(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::SelfS()
    };

    Ok(cre.push(this))
  }


  fn pre_bool(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Bool(start.into(), start.kind() == WK::True)
    };

    Ok(cre.push(this))
  }

  fn pre_number(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Number(start.into())
    };

    Ok(cre.push(this))
  }

  fn pre_string(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::String(start.into())
    };

    Ok(cre.push(this))
  }


  fn pre_tuple(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Unit
    if lex.peek()?.kind() == WK::ParenEnd {
      lex.bump()?;

      // Post
      let this = Expr{
        pos: lex.pos_extend(start),
        kind: ExprKind::Unit,
      };

      return Ok(cre.push(this))
    }
    
    let expr = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
    match lex.get_k()? {
      (WK::ParenEnd, _) => Ok(expr), // (X)

      (WK::Comma, _) => { // (X, ...)
        let mut vals = vec![ expr ];
        let mut is_tuple = false;
        
        loop {
          if lex.peek()?.kind() == WK::ParenEnd { lex.bump()?; break }
          
          vals.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);
          
          match lex.get_k()? {
            (WK::ParenEnd, _) => break,
            (WK::Comma, _) => { is_tuple = true; continue; }
            
            (_, c) => c.panic_kind2(WK::Comma, WK::ParenEnd)?
          }
        }
        
        if is_tuple || vals.len() != 1 {
          let rng = cre.extra(&vals);

          let this = Expr{
            pos: lex.pos_extend(start),
            kind: ExprKind::Tuple(rng)
          };
          
          Ok(cre.push(this))
        } else {
          Ok(vals[0])
        }
      }

      (_, c) => c.panic_kind2(WK::ParenEnd, WK::Comma)?
    }
  }

  fn pre_propagate(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    
    let (rng, prpg) = {
      let mut vals = vec![];
      let mut prpg = None;
      
      loop {
        if lex.peek()?.kind() == WK::SquareBracketEnd { lex.bump()?; break }
        
        vals.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);
        
        match lex.get_k()? {
          (WK::SquareBracketEnd, _) => break,
          (WK::Comma, _) => continue,

          (WK::Semicolon, _) => {
            prpg = Some(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);
            lex.get()?.expect_kind(WK::SquareBracketEnd)?;
            break
          }
          
          (_, c) => c.panic_kind2(WK::Comma, WK::SquareBracketEnd)?
        }
      }
      
      (cre.extra(&vals), prpg)
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: match prpg {
        None => ExprKind::Array(rng),
        Some(v) => ExprKind::Propagate(rng, v)
      }
    };
    
    Ok(cre.push(this))
  }


  fn pre_if(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let cond = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
    
    let then = Self::pre_block(ctx!(cre, sin, far, lex, sum))?;

    let elsb = match lex.peek_k()? {
      (WK::Ef, _) => Some(Self::pre_if(ctx!(cre, sin, far, lex, sum))?),

      (WK::Else, _) => {
        lex.bump()?;
        Some(Self::pre_block(ctx!(cre, sin, far, lex, sum))?)
      }
      _ => None,
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::If{ cond, then, elsb }
    };

    Ok(cre.push(this))
  }

  fn pre_match(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let cond = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
    
    let arms = {
      lex.get()?.expect_kind(WK::CurlyBracketBeg)?;
      
      let mut arm_ids = vec![];
      
      loop {
        if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break }
        
        let pat = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
        lex.get()?.expect_kind(WK::FatArrow)?;
        let body = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
        let arm = Thing::MatchArm(pat, body);
        arm_ids.push(cre.push(arm));
        
        match lex.peek_k()? {
          (WK::CurlyBracketEnd, _) => { lex.bump()?; break }
          (WK::Comma, _) => { lex.bump()?; }

          (_, c) => c.panic_kind2(WK::CurlyBracketEnd, WK::Comma)?
        }
      }

      cre.extra(&arm_ids)
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Match{ cond, arms }
    };

    Ok(cre.push(this))
  }

  fn pre_loop(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let blok = Self::pre_block(ctx!(cre, sin, far, lex, sum))?;

    let elsb = if lex.peek()?.kind() == WK::Else {
      lex.bump()?;
      Some(Self::read_expr(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };

    
    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Loop{ blok, elsb }
    };

    Ok(cre.push(this))
  }

  fn pre_while(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let cond = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
    
    let blok = Self::pre_block(ctx!(cre, sin, far, lex, sum))?;

    let elsb = if lex.peek()?.kind() == WK::Else {
      lex.bump()?;
      Some(Self::read_expr(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::While{ cond, blok, elsb }
    };

    Ok(cre.push(this))
  }

  fn pre_for(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let vars = PattParser::read_patt(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::In)?;
    
    let iter = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;

    let blok = Self::pre_block(ctx!(cre, sin, far, lex, sum))?;

    let elsb = if lex.peek()?.kind() == WK::Else {
      lex.bump()?;
      Some(Self::read_expr(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::ForIn{ vars, iter, blok, elsb }
    };

    Ok(cre.push(this))
  }


  fn pre_ret(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let label = if lex.peek()?.kind() == WK::Backtick {
      lex.bump()?;
      let name = match lex.get_k()? {
        (WK::Word, c) => c.ident(sin, far)?,
        (_, c) => return Err(Message::error(c, "expected identifier after `", &[lex.str(c)])),
      };
      Some(name.into())
    } else {
      None
    };

    let val = if lex.peek()?.kind() != WK::Semicolon {
      Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Return{ label, val }
    };

    Ok(cre.push(this))
  }

  fn pre_break(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let label = if lex.peek()?.kind() == WK::Backtick {
      lex.bump()?;
      let name = match lex.get_k()? {
        (WK::Word, c) => c.ident(sin, far)?,
        (_, c) => return Err(Message::error(c, "expected identifier after `", &[lex.str(c)])),
      };
      Some(name.into())
    } else {
      None
    };

    let val = if lex.peek()?.kind() != WK::Semicolon {
      Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Break{ label, val }
    };

    Ok(cre.push(this))
  }

  fn pre_continue(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let label = if lex.peek()?.kind() == WK::Backtick {
      lex.bump()?;
      let name = match lex.get_k()? {
        (WK::Word, c) => c.ident(sin, far)?,
        (_, c) => return Err(Message::error(c, "expected identifier after `", &[lex.str(c)])),
      };
      Some(name.into())
    } else {
      None
    };

    
    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Continue{ label }
    };

    Ok(cre.push(this))
  }


  fn pre_unsafe(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let blok = Self::pre_block(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Unsafe(blok)
    };

    Ok(cre.push(this))
  }

  fn pre_relaxed(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let blok = Self::pre_block(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Relaxed(blok)
    };

    Ok(cre.push(this))
  }


  fn pre_let(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let item = PattParser::read_patt(ctx!(cre, sin, far, lex, sum))?;

    let kind = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };

    let init = if lex.peek()?.kind() == WK::Assign {
      lex.bump()?;
      Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?)
    }
    else {
      None
    };


    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Let{ item, kind, init, ism: start.kind() == WK::Var }
    };

    Ok(cre.push(this))
  }


  fn pre_unary(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let op = Self::parse_unary_op(start.kind());
    
    let val = Self::read_expr_sub(ctx!(cre, sin, far, lex, sum), 85)?;
    

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Unary{ op, val }
    };

    Ok(cre.push(this))
  }
  

  fn pre_nick(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = start.ident(sin, far)?;

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Nick(name)
    };

    Ok(cre.push(this))
  }



  // Tool
  fn get_infix_bp(op: WK) -> Option<(u8, u8)> {
    use WK::*;

    match op {
      Assign | ArrowLeft | AssignmentAdd | AssignmentSub | AssignmentMul | AssignmentDiv | AssignmentRem |
      AssignmentBitwiseAnd | AssignmentBitwiseOr | AssignmentBitwiseXor | AssignmentLeftShift | AssignmentRighShift => Some((10, 11)),
      
      Dot2 => Some((15, 15)),
      
      LogicalOr  => Some((20, 21)),
      LogicalXor => Some((22, 23)),
      LogicalAnd => Some((24, 25)),
      
      Equal | NotEqual | AngleBeg | AngleEnd | BiggerEqual | SmallerEqual => Some((30, 31)),
      
      BitwiseOr  => Some((40, 41)),
      BitwiseXor => Some((42, 43)),
      BitwiseAnd => Some((44, 45)),
      
      ShiftLeft | ShiftRigh => Some((50, 51)),
      
      Add | Sub => Some((60, 61)),
      
      Mul | Div | Rem => Some((70, 71)),
      
      _ => None,
    }
  }

  fn parse_unary_op(op: WK) -> UnaryOp {
    match op {
      WK::Sub  => UnaryOp::Neg,
      WK::Add  => UnaryOp::Poz,
      WK::Bang => UnaryOp::Not,
      WK::BitwiseAnd => UnaryOp::Ref,
      WK::At => UnaryOp::Addr,
      _ => unreachable!("Unknown unary operator: {:?}", op),
    }
  }

  fn parse_binary_op(op: WK) -> BinOp {
    match op {
      WK::Add => BinOp::Bin(BinaryOp::Add),
      WK::Sub => BinOp::Bin(BinaryOp::Sub),
      WK::Mul => BinOp::Bin(BinaryOp::Mul),
      WK::Div => BinOp::Bin(BinaryOp::Div),
      WK::Rem => BinOp::Bin(BinaryOp::Rem),

      WK::Equal => BinOp::Bin(BinaryOp::Eq),
      WK::NotEqual => BinOp::Bin(BinaryOp::Ne),
      WK::AngleBeg => BinOp::Bin(BinaryOp::Lt),
      WK::AngleEnd => BinOp::Bin(BinaryOp::Gt),
      WK::SmallerEqual => BinOp::Bin(BinaryOp::Lte),
      WK::BiggerEqual => BinOp::Bin(BinaryOp::Gte),
      
      WK::LogicalAnd => BinOp::Bin(BinaryOp::And),
      WK::LogicalOr => BinOp::Bin(BinaryOp::Or),
      
      WK::AssignmentAdd => BinOp::AssignOp(BinaryOp::Add),
      WK::AssignmentSub => BinOp::AssignOp(BinaryOp::Sub),
      WK::AssignmentMul => BinOp::AssignOp(BinaryOp::Mul),
      WK::AssignmentDiv => BinOp::AssignOp(BinaryOp::Div),
      WK::AssignmentRem => BinOp::AssignOp(BinaryOp::Rem),
      WK::AssignmentBitwiseAnd => BinOp::AssignOp(BinaryOp::And),
      WK::AssignmentBitwiseOr  => BinOp::AssignOp(BinaryOp::Or),
      WK::AssignmentBitwiseXor => BinOp::AssignOp(BinaryOp::Xor),
      WK::AssignmentLeftShift  => BinOp::AssignOp(BinaryOp::Shl),
      WK::AssignmentRighShift  => BinOp::AssignOp(BinaryOp::Shr),
      
      WK::Assign => BinOp::Assign,
      WK::ArrowLeft => BinOp::Exchange,

      _ => unreachable!("unknown binary operator: {:?}", op),
    }
  }

}

enum BinOp {
  Bin(BinaryOp),
  AssignOp(BinaryOp),
  Assign,
  Exchange,
}
