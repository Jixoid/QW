use qwc_ast::{FieldKind, IdentSave, Item, Thing, Type, TypeId, TypeKind, Visibility, id::PushOkApi};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{Ctx, FieldParser, TypeParser, WordCheck, ctx, expr_p::ExprParser};


impl TypeParser {

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
          
          c @_ => todo!("{c:#?}") // TODO!
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

}
