use qwc_ast::{FunAttrs, IdentSave, Rng, Thing, Type, TypeId, TypeKind, Visibility};
use qwc_diagnostic::{Label, Message, msg::{DUPLICATE_ATTRIBUTE, EXPECTED_IDENTIFIER}};
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx, AttrParser, ExprParser, ItemParser, WordCheck};



pub struct TypeParser;

impl TypeParser {

  // Public
  pub fn read_type(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let attrs = AttrParser::read_attr(ctx!(cre, sin, far, lex, sum))?;
    
    let mut lhs = match lex.peek()?.kind() {
      WK::Question => Self::pre_option(ctx!(cre, sin, far, lex, sum))?,
      WK::Bang     => Self::pre_fail(ctx!(cre, sin, far, lex, sum))?,
      WK::Dot2     => Self::pre_range(ctx!(cre, sin, far, lex, sum))?,

      WK::BitwiseAnd => Self::pre_ref(ctx!(cre, sin, far, lex, sum))?,
      WK::BitwiseXor => Self::pre_ptr(ctx!(cre, sin, far, lex, sum))?,
      
      WK::SquareBracketBeg => Self::pre_slice_array_vector(ctx!(cre, sin, far, lex, sum))?,

      WK::ParenBeg => Self::pre_tuple(ctx!(cre, sin, far, lex, sum))?,

      WK::Fun => Self::pre_fun(ctx!(cre, sin, far, lex, sum), true)?,

      WK::SelfB => Self::pre_self(ctx!(cre, sin, far, lex, sum))?,
      WK::Type  => Self::pre_type(ctx!(cre, sin, far, lex, sum))?,

      _ => Self::pre_nick(ctx!(cre, sin, far, lex, sum))?
    };

    loop {
      lhs = match lex.peek()?.kind() {
        WK::Scope => Self::post_scope(ctx!(cre, sin, far, lex, sum), lhs)?,
        
        WK::AngleBeg => Self::post_specialize(ctx!(cre, sin, far, lex, sum), lhs)?,

        _ => break
      };
    }

    if let Some(a) = attrs { cre.attach(lhs, a); }

    Ok(lhs)
  }

  pub fn read_fun(ctx: &mut Ctx) -> Result<TypeId, Message> { Self::pre_fun(ctx, false) }


  // Ex
  fn post_scope(ctx: &mut Ctx, lhs: TypeId) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
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
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Path(rng)
    };

    Ok(cre.push(this))
  }

  fn post_specialize(ctx: &mut Ctx, lhs: TypeId) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
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
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Spec{ base: lhs, args }
    };

    Ok(cre.push(this))
  }



  // Sub
  fn pre_self(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::SelfT()
    };

    Ok(cre.push(this))
  }

  fn pre_type(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Type()
    };

    Ok(cre.push(this))
  }
  

  fn pre_option(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Option(sub)
    };

    Ok(cre.push(this))
  }

  fn pre_fail(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fail(sub)
    };

    Ok(cre.push(this))
  }

  fn pre_range(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Range(sub)
    };

    Ok(cre.push(this))
  }

  fn pre_ref(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let ism = if lex.peek()?.kind() == WK::Mut { lex.bump()?; true } else { false };

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Ref(sub, ism)
    };

    Ok(cre.push(this))
  }

  fn pre_ptr(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let ism = if lex.peek()?.kind() == WK::Mut { lex.bump()?; true } else { false };

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Ptr(sub, ism)
    };

    Ok(cre.push(this))
  }


  fn pre_fun(ctx: &mut Ctx, is_sub: bool) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = if is_sub { lex.get()? } else { lex.peek()? };

    let args = Self::read_fun_args(ctx!(cre, sin, far, lex, sum))?;

    let attr = {
      let mut attr: u8 = 0;
      
      while let (WK::Word, c) = lex.peek_k()? {
        let str = lex.str(c);
        match str {
          "static" => {
            lex.bump()?;
            if attr & FunAttrs::Static as u8 != 0 {
              sum.add(Message::warn(DUPLICATE_ATTRIBUTE, Label::new_pos(c)));
            }
            attr |= FunAttrs::Static as u8;
          }
          
          "const" => {
            lex.bump()?;
            if attr & FunAttrs::Const as u8 != 0 {
              sum.add(Message::warn(DUPLICATE_ATTRIBUTE, Label::new_pos(c)));
            }
            attr |= FunAttrs::Const as u8;
          }
          
          "pure" => {
            lex.bump()?;
            if attr & FunAttrs::Pure as u8 != 0 {
              sum.add(Message::warn(DUPLICATE_ATTRIBUTE, Label::new_pos(c)));
            }
            attr |= FunAttrs::Pure as u8;
          }
          
          _ => break
        }
      }

      attr
    };

    let ret = if lex.peek()?.kind() == WK::ArrowRigh {
      lex.bump()?;
      Some(Self::read_type(ctx!(cre, sin, far, lex, sum))?)
    } else {
      None
    };


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fun{ args, ret, attr }
    };

    Ok(cre.push(this))
  }


  fn pre_slice_array_vector(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum))?;

    let kind = match lex.get_k()? {
      (WK::SquareBracketEnd, _) => TypeKind::Slice(sub),

      (WK::Semicolon, _) => {
        let ext = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;

        lex.get()?.expect_kind(WK::SquareBracketEnd)?;

        TypeKind::Array(sub, ext)
      }

      (WK::Mul, _) => {
        let ext = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;

        lex.get()?.expect_kind(WK::SquareBracketEnd)?;

        TypeKind::Vector(sub, ext)
      }

      (_, c) => c.panic_kind3(WK::SquareBracketEnd, WK::Semicolon, WK::Mul)?
    };
    

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind
    };

    Ok(cre.push(this))
  }

  fn pre_tuple(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;

    // Unit
    if lex.peek()?.kind() == WK::ParenEnd {
      lex.bump()?;

      // Post
      let this = Type{
        pos: lex.pos_extend(start),
        kind: TypeKind::Unit,
      };

      return Ok(cre.push(this))
    }

    let kind = Self::read_type(ctx!(cre, sin, far, lex, sum))?;

    match lex.get_k()? {
      (WK::ParenEnd, _) => Ok(kind), // (X)

      (WK::Comma, _) => { // (X, ...)
        let mut vals = vec![ kind ];
        let mut is_tuple = false;
        
        loop {
          if lex.peek()?.kind() == WK::ParenEnd { lex.bump()?; break }
          
          vals.push(Self::read_type(ctx!(cre, sin, far, lex, sum))?);
          
          match lex.get_k()? {
            (WK::ParenEnd, _) => break,
            (WK::Comma, _) => { is_tuple = true; continue; }
            
            (_, c) => c.panic_kind2(WK::Comma, WK::ParenEnd)?
          }
        }
        
        if is_tuple || vals.len() != 1 {
          let rng = cre.extra(&vals);

          let this = Type{
            pos: lex.pos_extend(start),
            kind: TypeKind::Tuple(rng)
          };
          
          Ok(cre.push(this))
        } else {
          Ok(vals[0])
        }
      }

      (_, c) => c.panic_kind2(WK::ParenEnd, WK::Comma)?
    }
  }


  fn pre_nick(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.get()?;
    let name = start.ident(sin, far)?;

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Nick(name)
    };

    Ok(cre.push(this))
  }



  // Big
  pub fn sub_struct(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    let _bases = Self::read_bases(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let rng = {
      let mut ctn = vec![];

      loop {
        if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break }
        
        ctn.push( ItemParser::read_item_in(ctx!(cre, sin, far, lex, sum), &mut Visibility::Inherited)? );
      }

      cre.extra(&ctn)
    };

    
    // Post
    let this = Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Struct(rng)
    };

    Ok(cre.push(this))
  }

  pub fn sub_enum(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    
    lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let vals = {
      let mut vals = vec![];
      
      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (WK::CurlyBracketEnd, _) => break,
          
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };
        
        let item = if lex.peek()?.kind() == WK::Assign {
          lex.bump()?;
          let val = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;
          
          Thing::NamedExpr(name, val)
        } else {
          Thing::Name(name)
        };

        vals.push(cre.push(item));

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break },

          (WK::CurlyBracketEnd, _) => break,
          
          (_, c) => c.panic_kind2(WK::Comma, WK::CurlyBracketEnd)?,
        }
      }

      cre.extra(&vals)
    };

    
    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Enum(vals)
    };

    Ok(cre.push(this))
  }

  pub fn sub_flags(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    
    lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let vals = {
      let mut vals = vec![];
      
      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (WK::CurlyBracketEnd, _) => break,
          
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };
        
        let item = if lex.peek()?.kind() == WK::Assign {
          lex.bump()?;
          let val = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;
          
          Thing::NamedExpr(name, val)
        } else {
          Thing::Name(name)
        };

        vals.push(cre.push(item));

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break },

          (WK::CurlyBracketEnd, _) => break,
          
          (_, c) => c.panic_kind2(WK::Comma, WK::CurlyBracketEnd)?,
        }
      }

      cre.extra(&vals)
    };

    
    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Flags(vals)
    };

    Ok(cre.push(this))
  }

  pub fn sub_variant(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    
    lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let vals = {
      let mut vals = vec![];

      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (WK::CurlyBracketEnd, _) => break,
          
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };

        let item = if lex.peek()?.kind() == WK::ParenBeg {
          let payload_type = TypeParser::pre_tuple(ctx!(cre, sin, far, lex, sum))?;
          
          Thing::NamedType(name, payload_type)
        } else {
          Thing::Name(name)
        };

        vals.push(cre.push(item));

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break },
          (WK::CurlyBracketEnd, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::CurlyBracketEnd)?,
        }
      }

      cre.extra(&vals)
    };


    // Post
    let this = Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Variant(vals),
    };

    Ok(cre.push(this))
  }

  pub fn sub_iface(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    let _bases = Self::read_bases(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let rng = {
      let mut ctn = vec![];
      let defvis = &mut Visibility::Inherited;

      loop {
        if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break }
        
        ctn.push( ItemParser::read_item_in(ctx!(cre, sin, far, lex, sum), defvis)? );
      }

      cre.extra(&ctn)
    };

    
    // Post
    let this = Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Iface(rng),
    };

    Ok(cre.push(this))
  }

  pub fn sub_trait(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let start = lex.peek()?;
    let _bases = Self::read_bases(ctx!(cre, sin, far, lex, sum))?;

    lex.get()?.expect_kind(WK::CurlyBracketBeg)?;

    let rng = {
      let mut ctn = vec![];
      let defvis = &mut Visibility::Inherited;

      loop {
        if lex.peek()?.kind() == WK::CurlyBracketEnd { lex.bump()?; break }
        
        ctn.push( ItemParser::read_item_in(ctx!(cre, sin, far, lex, sum), defvis)? );
      }

      cre.extra(&ctn)
    };

    
    // Post
    let this = Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Trait(rng),
    };

    Ok(cre.push(this))
  }



  // Tool
  fn read_bases(ctx: &mut Ctx) -> Result<Rng, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;

      let mut arr = vec![];
      
      loop {
        arr.push( Self::read_type(ctx!(cre, sin, far, lex, sum))? );

        if lex.peek()?.kind() == WK::Comma { lex.bump()?; continue } else { break }
      }

      Ok(cre.extra(&arr))
    }
    else {
      Ok(Rng::empty())
    }
  }

  fn read_fun_args(ctx: &mut Ctx) -> Result<Rng, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    lex.get()?.expect_kind(WK::ParenBeg)?;
    
    let mut args = vec![];
    loop {
      if lex.peek()?.kind() == WK::ParenEnd { lex.bump()?; break; }
      
      let mut names = vec![];
      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c)))
        };
        names.push(name);

        match lex.get_k()? {
          (WK::Comma, _) => continue,
          (WK::Colon, _) => break,

          (_, c) => c.panic_kind2(WK::Colon, WK::Comma)?
        }
      }

      let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum))?;
      
      for x in names {
        let thing = Thing::NamedType(x, kind);

        args.push(cre.push(thing));
      }

      match lex.get_k()? {
        (WK::ParenEnd, _) => break,
        (WK::Comma, _) => continue,

        (_, c) => c.panic_kind2(WK::Comma, WK::ParenBeg)?
      }
    }

    Ok(cre.extra(&args))
  }

}
