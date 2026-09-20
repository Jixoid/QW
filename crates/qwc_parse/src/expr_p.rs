use qwc_ast::{BinaryOp, Expr, ExprId, ExprKind, IdentSave, Rng, Thing, UnaryOp};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx, PattParser, TypeParser, WordCheck};


pub struct ExprParser;

impl ExprParser {

  // Public
  pub fn read_expr(ctx: &mut Ctx) -> Result<ExprId, Message> {
    Self::read_expr_sub(ctx, 0)
  }

  fn read_expr_sub(ctx: &mut Ctx, min_bp: u8) -> Result<ExprId, Message> {
    let mut lhs = match ctx.lex.peek()?.kind() {
      WK::BraceL | WK::Backtick => Self::pre_block(ctx)?,

      WK::True | WK::False => Self::pre_bool(ctx)?,
      WK::Number => Self::pre_number(ctx)?,
      WK::String => Self::pre_string(ctx)?,

      WK::ParenL => Self::pre_tuple(ctx)?,
      WK::BracketL => Self::pre_propagate(ctx)?,

      WK::If    => Self::pre_if(ctx)?,
      WK::Match => Self::pre_match(ctx)?,
      WK::Loop  => Self::pre_loop(ctx)?,
      WK::While => Self::pre_while(ctx)?,
      WK::For   => Self::pre_for(ctx)?,
      
      WK::Ret      => Self::pre_ret(ctx)?,
      WK::Break    => Self::pre_break(ctx)?,
      WK::Continue => Self::pre_continue(ctx)?,
      
      WK::Unsafe  => Self::pre_unsafe(ctx)?,
      WK::Relaxed => Self::pre_relaxed(ctx)?,

      WK::SelfB => Self::pre_self_b(ctx)?,
      WK::SelfS => Self::pre_self_s(ctx)?,
      
      WK::Let | WK::Var => Self::pre_let(ctx)?,
      
      WK::Sub | WK::Add | WK::Bang => Self::pre_unary(ctx)?,

      _ => Self::pre_nick(ctx)?,
    };

    loop {
      lhs = match ctx.lex.peek()?.kind() {
        WK::Dot => Self::post_member(ctx, lhs)?,

        WK::Colon2 => Self::post_scope_spec(ctx, lhs)?,

        WK::ParenL   => Self::post_call(ctx, lhs)?,
        WK::BracketL => Self::post_index(ctx, lhs)?,

        WK::Bang2 | WK::Question |
        WK::Amp | WK::Caret => Self::post_unary(ctx, lhs)?,

        _ => break
      }
    }

    loop {
      let op_tok = ctx.lex.peek()?;
      
      let (_, r_bp) = match Self::get_infix_bp(op_tok.kind()) {
        Some((l, r)) if l >= min_bp => { ctx.lex.bump()?; (l, r) },
        _ => break,
      };

      let rhs = Self::read_expr_sub(ctx, r_bp)?;
      
      let kind = match Self::parse_binary_op(op_tok.kind()) {
        BinOp::Bin(op)      => ExprKind::Binary{op, lhs, rhs},
        BinOp::AssignOp(op) => ExprKind::AssignOp{op, lhs, rhs},

        BinOp::Assign   => ExprKind::Assign{lhs, rhs},
        BinOp::Exchange => ExprKind::Exchange{lhs, rhs},
      };

      let this = Expr{
        pos: ctx.lex.pos_extend(op_tok),
        kind 
      };

      lhs = ctx.cre.push(this);
    }

    Ok(lhs)
  }



  // Ex
  fn post_scope_spec(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    lex.get()?;

    match lex.peek()?.kind() {
      WK::Lt => Self::post_specialize(ctx!(cre, sin, far, lex, sum), lhs),
      _ => Self::post_scope(ctx!(cre, sin, far, lex, sum), lhs),
    }
  }


  fn post_member(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let rng = {
      let sub = Self::pre_nick(ctx!(cre, sin, far, lex, sum))?;
      
      let mut ctn = vec![lhs, sub];

      loop {
        match lex.peek_k()? {
          (WK::Dot, _) => {
            lex.bump()?;
            ctn.push(Self::pre_nick(ctx!(cre, sin, far, lex, sum))?);
          }

          (WK::Colon2, c) => return Err(Message::error(CANNOT_FIELD_ACCESS_AFTER_MEMBER, Label::new_pos(c))),

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
    let start = lex.peek()?;

    let rng = {
      let sub = Self::pre_nick(ctx!(cre, sin, far, lex, sum))?;
      
      let mut ctn = vec![lhs, sub];

      loop {
        if lex.peek()?.kind() == WK::Colon2 {
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

  fn post_specialize(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let args = if lex.peek()?.kind() == WK::Gt {
      lex.bump()?;
      Rng::empty()
    } else {
      let mut args = vec![];

      loop {
        let arg = match lex.peek_k()? {
          (WK::String | WK::Number | WK::BraceL, _) => ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?.to_any(),
          
          _ => TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?.to_any(),
        };
        
        args.push(arg);

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::Gt { lex.bump()?; break },
          
          (WK::Gt, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::Gt)?,
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


  fn post_call(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let args = if lex.peek()?.kind() == WK::ParenR {
      lex.bump()?;
      Rng::empty()
    } else {
      let mut args = vec![];

      loop {
        args.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break },
          
          (WK::ParenR, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?,
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

    let args = if lex.peek()?.kind() == WK::BracketR {
      lex.bump()?;
      Rng::empty()
    } else {
      let mut args = vec![];

      loop {
        args.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::BracketR { lex.bump()?; break },
          
          (WK::BracketR, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::BracketR)?,
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


  fn post_unary(ctx: &mut Ctx, lhs: ExprId) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let op = Self::parse_unary_op(start.kind());
    

    // Post
    let this = Expr{
      pos: lex.pos_extend(start),
      kind: ExprKind::Unary{ op, val: lhs }
    };

    Ok(cre.push(this))
  }
  
  

  // Sub
  pub fn pre_block(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let label = match start.kind() {
      WK::BraceL => None,

      WK::Backtick => {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
        };
        
        lex.get()?.expect_kind(WK::Colon)?;
        lex.get()?.expect_kind(WK::BraceL)?;
        
        Some(name.into())
      }
      
      _ => start.panic_kind2(WK::Backtick, WK::BraceL)?
    };

    let (rng, expr) = {
      let mut ctn = vec![];
      let mut expr = None;
      
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let ex_id = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
        match lex.peek_k()? {
          (WK::Semicolon, _) => { lex.bump()?; ctn.push(ex_id); },
          
          (WK::BraceR, _) => { expr = Some(ex_id); }
          
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
    if lex.peek()?.kind() == WK::ParenR {
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
      (WK::ParenR, _) => Ok(expr), // (X)

      (WK::Comma, _) => { // (X, ...)
        let mut vals = vec![ expr ];
        
        loop {
          if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break }
          
          vals.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);
          
          match lex.get_k()? {
            (WK::ParenR, _) => break,
            (WK::Comma, _) => continue,
            
            (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?
          }
        }
        
        let rng = cre.extra(&vals);


        // Post
        let this = Expr{
          pos: lex.pos_extend(start),
          kind: ExprKind::Tuple(rng)
        };
        
        Ok(cre.push(this))
      }

      (_, c) => c.panic_kind2(WK::ParenR, WK::Comma)?
    }
  }

  fn pre_propagate(ctx: &mut Ctx) -> Result<ExprId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    
    let (rng, prpg) = {
      let mut vals = vec![];
      let mut prpg = None;
      
      loop {
        if lex.peek()?.kind() == WK::BracketR { lex.bump()?; break }
        
        vals.push(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);
        
        match lex.get_k()? {
          (WK::BracketR, _) => break,
          (WK::Comma, _) => continue,

          (WK::Semicolon, _) => {
            prpg = Some(Self::read_expr(ctx!(cre, sin, far, lex, sum))?);
            lex.get()?.expect_kind(WK::BracketR)?;
            break
          }
          
          (_, c) => c.panic_kind2(WK::Comma, WK::BracketR)?
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
      lex.get()?.expect_kind(WK::BraceL)?;
      
      let mut arm_ids = vec![];
      
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let pat = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
        lex.get()?.expect_kind(WK::FatArrow)?;
        let body = Self::read_expr(ctx!(cre, sin, far, lex, sum))?;
        
        let arm = Thing::MatchArm(pat, body);
        arm_ids.push(cre.push(arm));
        
        match lex.peek_k()? {
          (WK::BraceR, _) => { lex.bump()?; break }
          (WK::Comma, _) => { lex.bump()?; }

          (_, c) => c.panic_kind2(WK::BraceR, WK::Comma)?
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
        (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
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
        (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
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
        (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
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

    let init = if lex.peek()?.kind() == WK::Eq {
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
      // Assign, Exchange
      Eq | ArrowLeft => Some((11, 10)),
      
      // Combinated Assign
      Lt2Eq | Gt2Eq |
      Amp2Eq | Pipe2Eq | Caret2Eq |
      AddEq | SubEq | MulEq | DivEq | RemEq => Some((11, 10)),

      // Logical
      Pipe2  => Some((20, 21)),
      Caret2 => Some((22, 23)),
      Amp2   => Some((24, 25)),
      
      // Equality
      Eq2 | BangEq => Some((30, 31)),
      
      // Size Comparisons
      Lt | Gt | LtEq | GtEq => Some((35, 36)),
      
      // Pipe Stream
      Pipe  => Some((40, 41)),
      
      // Range
      Dot2 | Dot2Eq => Some((45, 46)),
      
      // Shift
      Lt2 | Gt2 => Some((50, 51)),
      
      // Arithmetic
      Add | Sub => Some((60, 61)),
      Mul | Div | Rem => Some((70, 71)),
      
      _ => None,
    }
  }

  fn parse_unary_op(op: WK) -> UnaryOp {
    match op {
      // Prefix
      WK::Sub  => UnaryOp::Neg,
      WK::Add  => UnaryOp::Poz,
      WK::Bang => UnaryOp::Not,

      // Postfix
      WK::Question => UnaryOp::Try,
      WK::Bang2    => UnaryOp::Unwrap,
      WK::Amp      => UnaryOp::Ref,
      WK::Caret    => UnaryOp::Deref,

      _ => unreachable!("Unknown unary operator: {:?}", op),
    }
  }

  fn parse_binary_op(op: WK) -> BinOp {
    match op {
      // Arithmetic
      WK::Add => BinOp::Bin(BinaryOp::Add),
      WK::Sub => BinOp::Bin(BinaryOp::Sub),
      WK::Mul => BinOp::Bin(BinaryOp::Mul),
      WK::Div => BinOp::Bin(BinaryOp::Div),
      WK::Rem => BinOp::Bin(BinaryOp::Rem),
      
      WK::AddEq => BinOp::AssignOp(BinaryOp::Add),
      WK::SubEq => BinOp::AssignOp(BinaryOp::Sub),
      WK::MulEq => BinOp::AssignOp(BinaryOp::Mul),
      WK::DivEq => BinOp::AssignOp(BinaryOp::Div),
      WK::RemEq => BinOp::AssignOp(BinaryOp::Rem),

      // Shift
      WK::Lt2 => BinOp::Bin(BinaryOp::Shl),
      WK::Gt2 => BinOp::Bin(BinaryOp::Shr),

      WK::Lt2Eq => BinOp::AssignOp(BinaryOp::Shl),
      WK::Gt2Eq => BinOp::AssignOp(BinaryOp::Shr),

      // Comparison
      WK::Eq2    => BinOp::Bin(BinaryOp::Eq),
      WK::BangEq => BinOp::Bin(BinaryOp::Ne),
      
      WK::Lt => BinOp::Bin(BinaryOp::Lt),
      WK::Gt => BinOp::Bin(BinaryOp::Gt),
      WK::LtEq => BinOp::Bin(BinaryOp::LtEq),
      WK::GtEq => BinOp::Bin(BinaryOp::GtEq),
      
      // Logical
      WK::Amp2   => BinOp::Bin(BinaryOp::And),
      WK::Pipe2  => BinOp::Bin(BinaryOp::Or),
      WK::Caret2 => BinOp::Bin(BinaryOp::Xor),
      
      WK::Amp2Eq   => BinOp::AssignOp(BinaryOp::And),
      WK::Pipe2Eq  => BinOp::AssignOp(BinaryOp::Or),
      WK::Caret2Eq => BinOp::AssignOp(BinaryOp::Xor),

      // Pipe
      WK::Pipe => BinOp::Bin(BinaryOp::Pipe),
      
      // Special
      WK::Eq => BinOp::Assign,
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
