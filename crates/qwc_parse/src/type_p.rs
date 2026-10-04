use qwc_ast::{AnyRng, FieldKind, IdentSave, Item, Rng, Thing, ThingRng, Type, TypeId, TypeKind, TypeRng, Visibility, id::PushOkApi};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{AttrParser, ExprParser, WordCheck, ctx, FieldParser, parse::Ctx};



pub struct TypeParser;

impl TypeParser {

  // Public
  pub fn read_type(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let attrs = AttrParser::read_attr(ctx!(cre, sin, far, lex, sum, side))?;
    
    let mut lhs = match lex.peek()?.kind() {
      WK::Question => Self::pre_option(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Bang     => Self::pre_fail(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Dot2     => Self::pre_range(ctx!(cre, sin, far, lex, sum, side))?,

      WK::Amp   => Self::pre_ref(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Caret => Self::pre_ptr(ctx!(cre, sin, far, lex, sum, side))?,
      
      WK::BracketL => Self::pre_slice_array_vector(ctx!(cre, sin, far, lex, sum, side))?,

      WK::ParenL => Self::pre_tuple(ctx!(cre, sin, far, lex, sum, side))?,

      WK::Fun => Self::pre_fun(ctx!(cre, sin, far, lex, sum, side), true)?,

      WK::SelfB => Self::pre_self(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Type  => Self::pre_type(ctx!(cre, sin, far, lex, sum, side))?,

      _ => Self::pre_nick(ctx!(cre, sin, far, lex, sum, side))?
    };

    loop {
      lhs = match lex.peek()?.kind() {
        WK::Colon2 => Self::post_scope(ctx!(cre, sin, far, lex, sum, side), lhs)?,
        
        WK::Lt => Self::post_specialize(ctx!(cre, sin, far, lex, sum, side), lhs)?,

        _ => break
      };
    }

    if let Some(a) = attrs { cre.attach(lhs, a); }

    Ok(lhs)
  }

  pub fn read_fun(ctx: &mut Ctx) -> Result<TypeId, Message> { Self::pre_fun(ctx, false) }


  // Ex
  fn post_scope(ctx: &mut Ctx, lhs: TypeId) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let rng = {
      let sub = Self::pre_nick(ctx!(cre, sin, far, lex, sum, side))?;
      
      let mut ctn = vec![lhs, sub];

      loop {
        if lex.peek()?.kind() == WK::Colon2 {
          lex.bump()?;

          ctn.push(Self::pre_nick(ctx!(cre, sin, far, lex, sum, side))?);
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

  fn post_specialize(ctx: &mut Ctx, lhs: TypeId) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let args = if lex.peek()?.kind() == WK::Gt {
      lex.bump()?;
      AnyRng::empty()
    } else {
      let mut args = vec![];

      loop {
        let arg = match lex.peek_k()? {
          (WK::String | WK::Number | WK::BraceL, _) => ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?.to_any(),
          
          _ => TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?.to_any(),
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
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Spec{ base: lhs, args }
    };

    Ok(cre.push(this))
  }



  // Sub
  fn pre_self(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::SelfT,
    };

    Ok(cre.push(this))
  }

  fn pre_type(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Type,
    };

    Ok(cre.push(this))
  }
  

  fn pre_option(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Option(sub)
    };

    Ok(cre.push(this))
  }

  fn pre_fail(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fail(sub)
    };

    Ok(cre.push(this))
  }

  fn pre_range(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Range(sub)
    };

    Ok(cre.push(this))
  }

  fn pre_ref(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let ism = if lex.peek()?.kind() == WK::Mut { lex.bump()?; true } else { false };

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Ref(sub, ism)
    };

    Ok(cre.push(this))
  }

  fn pre_ptr(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let ism = if lex.peek()?.kind() == WK::Mut { lex.bump()?; true } else { false };

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Ptr(sub, ism)
    };

    Ok(cre.push(this))
  }


  fn pre_fun(ctx: &mut Ctx, is_sub: bool) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = if is_sub { lex.get()? } else { lex.peek()? };

    let (self_kind, args) = Self::read_fun_args(ctx!(cre, sin, far, lex, sum, side))?;

    let ret = if lex.peek()?.kind() == WK::ArrowRight {
      lex.bump()?;
      Some(Self::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };


    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fun{ self_kind, args, ret, attr: 0 }
    };

    Ok(cre.push(this))
  }


  fn pre_slice_array_vector(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;

    let kind = match lex.get_k()? {
      (WK::BracketR, _) => TypeKind::Slice(sub),

      (WK::Semicolon, _) => {
        let ext = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

        lex.get()?.expect_kind(WK::BracketR)?;

        TypeKind::Array(sub, ext)
      }

      (WK::Mul, _) => {
        let ext = if lex.peek()?.kind() == WK::BracketR { None } else { Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?) };

        lex.get()?.expect_kind(WK::BracketR)?;

        if let Some(ext) = ext {
          TypeKind::Vector(sub, ext)
        } else {
          TypeKind::VScale(sub)
        }
      }

      (_, c) => c.panic_kind3(WK::BracketR, WK::Semicolon, WK::Mul)?
    };
    

    // Post
    let this = Type{
      pos: lex.pos_extend(start),
      kind
    };

    Ok(cre.push(this))
  }

  fn pre_tuple(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    // Unit
    if lex.peek()?.kind() == WK::ParenR {
      lex.bump()?;

      // Post
      let this = Type{
        pos: lex.pos_extend(start),
        kind: TypeKind::Unit,
      };

      return Ok(cre.push(this))
    }

    let kind = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;

    match lex.get_k()? {
      (WK::ParenR, _) => Ok(kind), // (X)

      (WK::Comma, _) => { // (X, ...)
        let mut vals = vec![ kind ];
        let mut is_tuple = false;
        
        loop {
          if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break }
          
          vals.push(Self::read_type(ctx!(cre, sin, far, lex, sum, side))?);
          
          match lex.get_k()? {
            (WK::ParenR, _) => break,
            (WK::Comma, _) => { is_tuple = true; continue; }
            
            (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?
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

      (_, c) => c.panic_kind2(WK::ParenR, WK::Comma)?
    }
  }


  fn pre_nick(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
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
  pub fn sub_struct(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek()?;
    let _bases = Self::read_bases(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get()?.expect_kind(WK::BraceL)?;

    let (rng, impin) = {
      let mut ctn = vec![];
      let mut impin = vec![];

      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }

        let id = FieldParser::read_field(ctx!(cre, sin, far, lex, sum, side), &mut Visibility::Inherited)?;
        let it = cre.get(id);
        match it.kind {
          FieldKind::MemberVar {kind} => ctn.push(Thing::NamedType(it.name.unwrap(), kind).push(cre)),
          
          FieldKind::ImplIn {trait_ty, ctn} => impin.push((trait_ty, ctn, it.pos, it.vis)),
          
          _ => panic!()
        }
      }

      (cre.extra(&ctn), impin.into_boxed_slice())
    };

    let type_ty = Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Struct(rng)
    }.push(ctx.cre);

    
    for (trait_ty, ctn, pos, vis) in impin {
      let sideimp = Item {
        kind: qwc_ast::ItemKind::Impl { type_ty, trait_ty: Some(trait_ty), ctn },
        name: None,
        pos,
        vis,
      }.push(ctx.cre);

      ctx.side.push(sideimp);
    }

    Ok(type_ty)
  }

  pub fn sub_enum(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek()?;
    
    lex.get()?.expect_kind(WK::BraceL)?;

    let vals = {
      let mut vals = vec![];
      
      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (WK::BraceR, _) => break,
          
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };
        
        let item = if lex.peek()?.kind() == WK::Eq {
          lex.bump()?;
          let val = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
          
          Thing::NamedExpr(name, val)
        } else {
          Thing::Name(name)
        };

        vals.push(cre.push(item));

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break },

          (WK::BraceR, _) => break,
          
          (_, c) => c.panic_kind2(WK::Comma, WK::BraceR)?,
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

  pub fn sub_flags(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek()?;
    
    lex.get()?.expect_kind(WK::BraceL)?;

    let vals = {
      let mut vals = vec![];
      
      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (WK::BraceR, _) => break,
          
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };
        
        let item = if lex.peek()?.kind() == WK::Eq {
          lex.bump()?;
          let val = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
          
          Thing::NamedExpr(name, val)
        } else {
          Thing::Name(name)
        };

        vals.push(cre.push(item));

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break },

          (WK::BraceR, _) => break,
          
          (_, c) => c.panic_kind2(WK::Comma, WK::BraceR)?,
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

  pub fn sub_variant(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek()?;
    
    lex.get()?.expect_kind(WK::BraceL)?;

    let vals = {
      let mut vals = vec![];

      loop {
        let name = match lex.get_k()? {
          (WK::Word, c) => c.ident(sin, far)?,
          (WK::BraceR, _) => break,
          
          (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };

        let item = if lex.peek()?.kind() == WK::ParenL {
          let payload_type = TypeParser::pre_tuple(ctx!(cre, sin, far, lex, sum, side))?;
          
          Thing::NamedType(name, payload_type)
        } else {
          Thing::Name(name)
        };

        vals.push(cre.push(item));

        match lex.get_k()? {
          (WK::Comma, _) => if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break },
          (WK::BraceR, _) => break,

          (_, c) => c.panic_kind2(WK::Comma, WK::BraceR)?,
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

  pub fn sub_iface(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek()?;
    let _bases = Self::read_bases(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get()?.expect_kind(WK::BraceL)?;

    let rng = {
      let mut ctn = vec![];
      let defvis = &mut Visibility::Inherited;

      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        ctn.push( FieldParser::read_field(ctx!(cre, sin, far, lex, sum, side), defvis)? );
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

  pub fn sub_trait(ctx: &mut Ctx) -> Result<TypeId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek()?;
    let _bases = Self::read_bases(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get()?.expect_kind(WK::BraceL)?;

    let rng = {
      let mut ctn = vec![];
      let defvis = &mut Visibility::Inherited;

      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        ctn.push( FieldParser::read_field(ctx!(cre, sin, far, lex, sum, side), defvis)? );
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
  fn read_bases(ctx: &mut Ctx) -> Result<TypeRng, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;

      let mut arr = vec![];
      
      loop {
        arr.push( Self::read_type(ctx!(cre, sin, far, lex, sum, side))? );

        if lex.peek()?.kind() == WK::Comma { lex.bump()?; continue } else { break }
      }

      Ok(cre.extra(&arr))
    }
    else {
      Ok(Rng::empty())
    }
  }

  fn read_fun_args(ctx: &mut Ctx) -> Result<(Option<TypeId>, ThingRng), Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    lex.get()?.expect_kind(WK::ParenL)?;
    
    let mut args = vec![];
    let mut self_kind = None;

    if lex.peek()?.kind() != WK::ParenR {
      if lex.peek()?.kind() == WK::Amp {
        let amp_word = lex.get()?;
        let ism = if lex.peek()?.kind() == WK::Mut {
          lex.bump()?;
          true
        } else {
          false
        };
        let self_word = lex.get()?;
        if self_word.kind() != WK::SelfS {
          return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(self_word)));
        }
        let self_t = cre.push(Type {
          pos: lex.pos_extend(self_word),
          kind: TypeKind::SelfT,
        });
        let ref_t = cre.push(Type {
          pos: lex.pos_extend(amp_word),
          kind: TypeKind::Ref(self_t, ism),
        });
        self_kind = Some(ref_t);
      } else if lex.peek()?.kind() == WK::Mut {
        let mut_word = lex.get()?;
        if lex.peek()?.kind() == WK::SelfS {
          let self_word = lex.get()?;
          let self_t = cre.push(Type {
            pos: lex.pos_extend(self_word),
            kind: TypeKind::SelfT,
          });
          self_kind = Some(self_t);
        } else {
          return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(mut_word)));
        }
      } else if lex.peek()?.kind() == WK::SelfS {
        let self_word = lex.get()?;
        let self_t = cre.push(Type {
          pos: lex.pos_extend(self_word),
          kind: TypeKind::SelfT,
        });
        self_kind = Some(self_t);
      }

      if self_kind.is_some() {
        match lex.peek_k()? {
          (WK::Comma, _) => { lex.bump()?; }
          (WK::ParenR, _) => {
            lex.bump()?;
            return Ok((self_kind, cre.extra(&args)));
          }
          (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?
        }
      }
    }

    loop {
      if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break; }
      
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

      let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
      
      for x in names {
        let thing = Thing::NamedType(x, kind);

        args.push(cre.push(thing));
      }

      match lex.get_k()? {
        (WK::ParenR, _) => break,
        (WK::Comma, _) => continue,

        (_, c) => c.panic_kind2(WK::Comma, WK::ParenL)?
      }
    }

    Ok((self_kind, cre.extra(&args)))
  }

}
