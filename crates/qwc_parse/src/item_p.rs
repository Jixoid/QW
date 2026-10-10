/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{IdentSave, Item, ItemId, ItemKind, Rng, Thing, ThingId, ThingKind, Visibility};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{ExprParser, MetaParser, TypeParser, WordCheck, ctx, field_p::FieldParser, parse::Ctx};


pub struct ItemParser;

impl ItemParser {

  // Public
  pub fn read_item(ctx: &mut Ctx, defvis: &mut Visibility) -> Result<ItemId, Message> {
    let (vis, attr) = MetaParser::read_start(ctx, defvis)?;

    let id = match ctx.lex.peek_k()? {
      (WK::Let |
       WK::Var, _)     => Self::sub_let     (ctx, vis)?,
      (WK::Using, _)   => Self::sub_using   (ctx, vis)?,
      (WK::Fun, _)     => Self::sub_fun     (ctx, vis)?,
      (WK::Task, _)    => Self::sub_task    (ctx, vis)?,
      (WK::Impl, _)    => Self::sub_impl    (ctx, vis)?,
      (WK::Generic, _) => Self::sub_generic (ctx, vis)?,
      (WK::Use, _)     => Self::sub_use     (ctx, vis)?,
      (WK::Mod, _)     => Self::sub_mod     (ctx, vis)?,
      
      (WK::Struct | WK::Iface | WK::Trait | WK::Enum | WK::Flags | WK::Variant, _) => Self::sub_itemty(ctx, vis)?,

      (_, c) => return Err(Message::error(UNKNOWN_KEYWORD, Label::new_pos(c))),
    };

    if let Some(a) = attr { ctx.cre.attach(id, a); }
    
    Ok(id)
  }


  // Sub
  fn sub_let(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };

    lex.get()?.expect_kind(WK::Eq)?;

    let value = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get()?.expect_kind(WK::Semicolon)?;
    

    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Let{kind, value, ism: start.kind() == WK::Var}
    };
    
    Ok(cre.push(this))
  }

  fn sub_using(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;
    
    lex.get()?.expect_kind(WK::Eq)?;
    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
    lex.get()?.expect_kind(WK::Semicolon)?;

    
    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Using(kind),
    }; 

    Ok(cre.push(this))
  }
  
  fn sub_fun(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum, side))?;

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Fun{ kind, blok },
    };
    
    Ok(cre.push(this))
  }

  fn sub_task(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let kind = TypeParser::read_task(ctx!(cre, sin, far, lex, sum, side))?;

    let blok = match lex.peek_k()? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump()?; None },

      (_, c) => c.panic_kind2(WK::BraceL, WK::Semicolon)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::Task{ kind, blok },
    };
    
    Ok(cre.push(this))
  }

  fn sub_impl(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    let type_ty = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
    
    let trait_ty = if lex.peek()?.kind() == WK::Colon {
      lex.bump()?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };

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
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::Impl{ type_ty, trait_ty, ctn: rng },
    };
    
    Ok(cre.push(this))
  }
  
  fn sub_generic(ctx: &mut Ctx, mut vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    lex.get()?.expect_kind(WK::Lt)?;
    
    let params = {
      let mut args = vec![];

      loop {
        if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break; }
        
        let mut names = vec![];
        let hty = loop {
          let name = match lex.get_k()? {
            (WK::Word, c) => c.ident(sin, far)?,
            (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c)))
          };
          names.push(name);

          match lex.peek_k()? {
            (WK::Comma, _) => {lex.bump()?; continue},
            (WK::Colon, _) => {lex.bump()?; break true},
            (WK::Semicolon, _) => {lex.bump()?; break false},
            (WK::Gt, _) => break false,

            (_, c) => c.panic_kind4(WK::Colon, WK::Comma, WK::Gt, WK::Semicolon)?
          }
        };

        let kind = if hty { Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?) } else { None };
        
        for x in names {
          let thing = match kind {
            Some(kind) => Thing{kind: ThingKind::NamedType(x, kind), pos: x.into()},
            None => Thing{kind: ThingKind::Name(x), pos: x.into()},
          };

          args.push(cre.push(thing));
        }

        match lex.get_k()? {
          (WK::Gt, _) => break,
          (WK::Comma, _) => continue,

          (_, c) => c.panic_kind2(WK::Comma, WK::Gt)?
        }
      }

      cre.extra(&args)
    };

    let reqs = if { let p = lex.peek()?; p.kind() == WK::Word && lex.str(p) == "requires" } {
      lex.bump()?;
      let mut reqs = vec![];
      
      loop {
        let name = if let (WK::Word, c) = lex.peek_k()? { lex.bump()?; c.ident(sin, far)? } else { break };
        
        lex.get()?.expect_kind(WK::Colon)?;
        
        let mut tys = vec![];
        
        loop {
          tys.push(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?);
          
          match lex.get_k()? {
            (WK::Pipe, _) => continue,
            (WK::Semicolon, _) => break,

            (_, c) => c.panic_kind2(WK::Pipe, WK::Semicolon)?
          }
        }
        
        let rng = cre.extra(&tys);
        
        let this = Thing{kind: ThingKind::NamedTypeList(name, rng), pos: name.into()};
        
        reqs.push(cre.push(this));
      }
      
      cre.extra(&reqs)
    } else {
      Rng::empty()
    };
    
    let ctn = if lex.peek()?.kind() == WK::BraceL {
      lex.bump()?;

      let mut ctn = vec![];
      loop {
        if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
        
        let id = Self::read_item(ctx!(cre, sin, far, lex, sum, side), &mut vis)?;

        ctn.push(id);
      }

      cre.extra(&ctn)
    } else {
      let id = Self::read_item(ctx!(cre, sin, far, lex, sum, side), &mut vis)?;

      cre.extra(&[id])
    };
    

    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::Generic{ params, reqs, ctn },
    };

    Ok(cre.push(this))
  }

  fn sub_use(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let begin = match lex.get_k()? {
      (WK::Crate, c) => cre.push(Thing{kind: ThingKind::Crate, pos: c.into()}),
      (WK::Super, c) => cre.push(Thing{kind: ThingKind::Super, pos: c.into()}),
      (WK::Word, c)  => { let a = c.ident(sin, far)?; cre.push(Thing{kind: ThingKind::Name(a), pos: a.into()}) },

      (_, c) => return Err(Message::error(UNKNOWN_USE_STARTER, Label::new_pos(c))),
    };

    let rng = {      
      let mut all = vec![ begin ];
      
      loop {
        match lex.get_k()? {
          (WK::Semicolon, _) => break,
          (WK::Colon2, _) => all.push(Self::read_use_sub(ctx!(cre, sin, far, lex, sum, side))?),
          (WK::As, _) => {
            let alias = lex.get()?.ident(sin, far)?;
            let last_id = all.pop().unwrap();
            all.push(cre.push(Thing{kind: ThingKind::Alias(last_id, alias), pos: alias.into()}));
          }
        
          (_, c) => c.panic_kind2(WK::Colon2, WK::Semicolon)?,
        }
      }

      cre.extra(&all)
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: ItemKind::Import( rng )
    };

    Ok(cre.push(this))
  }

  fn read_use_sub(ctx: &mut Ctx) -> Result<ThingId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let ret = match lex.get_k()? {
      (WK::Crate, c) => cre.push(Thing{kind: ThingKind::Crate, pos: c.into()}),
      (WK::Super, c) => cre.push(Thing{kind: ThingKind::Super, pos: c.into()}),
      (WK::Mul, c)   => cre.push(Thing{kind: ThingKind::Wildcard, pos: c.into()}),

      (WK::Word, c) => { let a = c.ident(sin, far)?; cre.push(Thing{kind: ThingKind::Name(a), pos: a.into()}) },

      (_, c) => return Err(Message::error(UNKNOWN_USE_SEGMENT, Label::new_pos(c))),
    };

    Ok(ret)
  }

  fn sub_mod(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;
    let name = lex.get()?.ident(sin, far)?;

    let ctn = match lex.get_k()? {
      (WK::Semicolon, _) => None,

      (WK::BraceL, _) => {
        let mut ctn = vec![];
        let defvis = &mut Visibility::Inherited;

        loop {
          if lex.peek()?.kind() == WK::BraceR { lex.bump()?; break }
          
          let id = Self::read_item(ctx!(cre, sin, far, lex, sum, side), defvis)?;

          ctn.push(id);
        }

        Some(cre.extra(&ctn))
      }

      (_, c) => c.panic_kind2(WK::Semicolon, WK::BraceL)?
    };


    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: match ctn {
        None => ItemKind::ModuleUnloaded,
        Some(v) => ItemKind::Module(v),
      }
    };
    
    Ok(cre.push(this))
  }

  fn sub_itemty(ctx: &mut Ctx, vis: Visibility) -> Result<ItemId, Message> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get()?;

    let name = lex.get()?.ident(sin, far)?;
    
    let kind = match start.kind() {
      WK::Struct  => TypeParser::sub_struct(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Iface   => TypeParser::sub_iface(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Trait   => TypeParser::sub_trait(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Enum    => TypeParser::sub_enum(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Flags   => TypeParser::sub_flags(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Variant => TypeParser::sub_variant(ctx!(cre, sin, far, lex, sum, side))?,

      _ => panic!(),
    };
    
    
    // Post
    let this = Item{
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: ItemKind::ItemTy(kind),
    }; 

    Ok(cre.push(this))
  }

}
