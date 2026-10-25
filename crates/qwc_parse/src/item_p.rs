/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{DefPath, DefPathId, IdentSave, Item, ItemId, ItemKind, PushOkApi, Rng, Thing, ThingId, ThingKind, Visibility};
use qwc_diagnostic::{ret_err, Label, Message, Status::{self, Success}, msg::*};
use qwc_lexer::{WK, Word};

use crate::{ExprParser, MetaParser, TypeParser, WordCheck, ctx, field_p::FieldParser, parse::Ctx};


pub struct ItemParser;

impl ItemParser {

  // Public
  pub fn read_item(ctx: &mut Ctx, base: DefPathId, defvis: &mut Visibility) -> Status<ItemId> {
    let (vis, attr) = MetaParser::read_start(ctx, defvis)?;

    let id = match ctx.lex.peek_k(ctx.sum)? {
      (WK::Let |
       WK::Var, _)     => Self::sub_let     (ctx, base, vis)?,
      (WK::Using, _)   => Self::sub_using   (ctx, base, vis)?,
      (WK::Fun, _)     => Self::sub_fun     (ctx, base, vis)?,
      (WK::Task, _)    => Self::sub_task    (ctx, base, vis)?,
      (WK::Impl, _)    => Self::sub_impl    (ctx, base, vis)?,
      (WK::Generic, _) => Self::sub_generic (ctx, base, vis)?,
      (WK::Use, _)     => Self::sub_use     (ctx, base, vis)?,
      (WK::Mod, _)     => Self::sub_mod     (ctx, base, vis)?,
      
      (WK::Struct | WK::Iface | WK::Trait | WK::Enum | WK::Flags | WK::Variant, _) => Self::sub_itemty(ctx, base, vis)?,

      (_, c) => ret_err!(ctx.sum, Message::error(UNKNOWN_KEYWORD, Label::new_pos(c))),
    };

    if let Some(a) = attr { ctx.cre.attach(id, a); }
    
    Success(id)
  }


  // Sub
  fn sub_let(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let kind = if lex.peek(sum)?.kind() == WK::Colon {
      lex.bump(sum)?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };

    lex.get(sum)?.expect_kind(sum, WK::Eq)?;

    let value = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get(sum)?.expect_kind(sum, WK::Semicolon)?;

    let path = DefPath::Path { base, name: name.sid() }.push(cre);


    // Post
    Item {
      kind: ItemKind::Let{kind, value, ism: start.kind() == WK::Var},
      name: Some(name),
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }

  fn sub_using(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;
    
    lex.get(sum)?.expect_kind(sum, WK::Eq)?;
    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
    lex.get(sum)?.expect_kind(sum, WK::Semicolon)?;

    let path = DefPath::Path { base, name: name.sid() }.push(cre);

    
    // Post
    Item {
      kind: ItemKind::Using(kind),
      name: Some(name),
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }
  
  fn sub_fun(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum, side))?;

    let blok = match lex.peek_k(sum)? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump(sum)?; None },

      (_, c) => c.panic_kind2(sum, WK::BraceL, WK::Semicolon)?
    };

    let path = DefPath::Path { base, name: name.sid() }.push(cre);


    // Post
    Item {
      kind: ItemKind::Fun{ kind, blok },
      name: Some(name),
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }

  fn sub_task(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let kind = TypeParser::read_task(ctx!(cre, sin, far, lex, sum, side))?;

    let blok = match lex.peek_k(sum)? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump(sum)?; None },

      (_, c) => c.panic_kind2(sum, WK::BraceL, WK::Semicolon)?
    };

    let path = DefPath::Path { base, name: name.sid() }.push(cre);


    // Post
    Item {
      kind: ItemKind::Task{ kind, blok },
      name: Some(name),
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }

  fn sub_impl(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let type_ty = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
    
    let trait_ty = if lex.peek(sum)?.kind() == WK::Colon {
      lex.bump(sum)?;
      Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };
    
    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;

    let rng = {
      let mut ctn = vec![];
      let defvis = &mut Visibility::Inherited;

      loop {
        if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }
        
        ctn.push( FieldParser::read_field(ctx!(cre, sin, far, lex, sum, side), defvis)? );
      }

      cre.extra(&ctn)
    };

    let path = DefPath::Impl(base).push(cre);


    // Post
    Item {
      kind: ItemKind::Impl{ type_ty, trait_ty, ctn: rng },
      name: None,
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }
  
  fn sub_generic(ctx: &mut Ctx, base: DefPathId, mut vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    lex.get(sum)?.expect_kind(sum, WK::Lt)?;

    let path = DefPath::Generic(base).push(cre);
    
    let params = {
      let mut args = vec![];

      loop {
        if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break; }
        
        let mut names = vec![];
        let hty = loop {
          let name = match lex.get_k(sum)? {
            (WK::Word, c) => c.ident(sum, sin, far)?,
            (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
          };
          names.push(name);

          match lex.peek_k(sum)? {
            (WK::Comma, _) => {lex.bump(sum)?; continue},
            (WK::Colon, _) => {lex.bump(sum)?; break true},
            (WK::Semicolon, _) => {lex.bump(sum)?; break false},
            (WK::Gt, _) => break false,

            (_, c) => c.panic_kind4(sum, WK::Colon, WK::Comma, WK::Gt, WK::Semicolon)?
          }
        };

        let kind = if hty { Some(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?) } else { None };
        
        for x in names {
          let thing = match kind {
            Some(kind) => Thing{kind: ThingKind::NamedType(x, kind), pos: x.into()},
            None => Thing{kind: ThingKind::Name(x), pos: x.into()},
          };

          args.push(thing.push(cre));
        }

        match lex.get_k(sum)? {
          (WK::Gt, _) => break,
          (WK::Comma, _) => continue,

          (_, c) => c.panic_kind2(sum, WK::Comma, WK::Gt)?
        }
      }

      cre.extra(&args)
    };

    let reqs = if lex.peek(sum)?.kind() == WK::Reqs {
      lex.bump(sum)?;
      let mut reqs = vec![];
      
      while let (WK::Word, c) = lex.peek_k(sum)? {
        let name = { lex.bump(sum)?; c.ident(sum, sin, far)? };
        
        lex.get(sum)?.expect_kind(sum, WK::Colon)?;
        
        let mut tys = vec![];
        
        loop {
          tys.push(TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?);
          
          match lex.get_k(sum)? {
            (WK::Add, _) => continue,
            (WK::Comma, _) => break,

            (_, c) => c.panic_kind2(sum, WK::Add, WK::Comma)?
          }
        }
        
        let rng = cre.extra(&tys);
        
        let this = Thing{kind: ThingKind::NamedTypeList(name, rng), pos: name.into()};
        
        reqs.push(this.push(cre));
      }
      
      cre.extra(&reqs)
    } else {
      Rng::empty()
    };
    
    let ctn = {
      let mut ctn = vec![];
      let mut side = vec![];

      if lex.peek(sum)?.kind() == WK::BraceL {
        lex.bump(sum)?;
        
        loop {
          if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }
          
          let id = Self::read_item(ctx!(cre, sin, far, lex, sum, &mut side), path, &mut vis)?;
          
          ctn.push(id);
        }
        
      } else {
        let id = Self::read_item(ctx!(cre, sin, far, lex, sum, &mut side), path, &mut vis)?;
        
        ctn.push(id);
      }

      ctn.extend(side);

      cre.extra(&ctn)
    };
    

    // Post
    Item {
      kind: ItemKind::Generic{ params, reqs, ctn },
      name: None,
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }

  fn sub_use(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let begin = match lex.get_k(sum)? {
      (WK::Crate, c) => Thing{kind: ThingKind::Crate, pos: c.into()}.push(cre),
      (WK::Super, c) => Thing{kind: ThingKind::Super, pos: c.into()}.push(cre),
      (WK::SelfS, c) => Thing{kind: ThingKind::Self_, pos: c.into()}.push(cre),
      (WK::Word, c)  => { let a = c.ident(sum, sin, far)?; Thing{kind: ThingKind::Name(a), pos: a.into()}.push(cre) },
      (WK::BraceL, c) => Self::read_use_list(ctx!(cre, sin, far, lex, sum, side), c)?,

      (_, c) => ret_err!(sum, Message::error(UNKNOWN_USE_STARTER, Label::new_pos(c))),
    };

    let rng = {      
      let mut all = vec![ begin ];
      
      loop {
        match lex.get_k(sum)? {
          (WK::Semicolon, _) => break,
          (WK::Colon2, c) => {
            let last_kind = all.last().map(|&id| cre.get(id).kind);
            if matches!(last_kind, Some(ThingKind::List(..) | ThingKind::Wildcard)) {
              ret_err!(sum, Message::error(UNKNOWN_USE_SEGMENT, Label::new_pos(c)));
            }
            all.push(Self::read_use_sub(ctx!(cre, sin, far, lex, sum, side))?);
          }
          (WK::As, c) => {
            let last_kind = all.last().map(|&id| cre.get(id).kind);
            if matches!(last_kind, Some(ThingKind::List(..) | ThingKind::Wildcard)) {
              ret_err!(sum, Message::error(UNKNOWN_USE_SEGMENT, Label::new_pos(c)));
            }
            let alias = lex.get(sum)?.ident(sum, sin, far)?;
            let last_id = all.pop().unwrap();
            all.push(Thing{kind: ThingKind::Alias(last_id, alias), pos: alias.into()}.push(cre));
          }
        
          (_, c) => c.panic_kind2(sum, WK::Colon2, WK::Semicolon)?,
        }
      }

      cre.extra(&all)
    };

    let path = DefPath::Use(base).push(cre);


    // Post
    Item {
      kind: ItemKind::Import( rng ),
      name: None,
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }

  fn read_use_list(ctx: &mut Ctx, start: Word) -> Status<ThingId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let mut items = vec![];

    loop {
      if lex.peek(sum)?.kind() == WK::BraceR {
        lex.bump(sum)?;
        break;
      }

      let item = Self::read_use_tree(ctx!(cre, sin, far, lex, sum, side))?;
      items.push(item);

      match lex.get_k(sum)? {
        (WK::Comma, _) => {
          if lex.peek(sum)?.kind() == WK::BraceR {
            lex.bump(sum)?;
            break;
          }
          continue;
        }
        (WK::BraceR, _) => break,
        (_, c) => c.panic_kind2(sum, WK::Comma, WK::BraceR)?,
      }
    }

    let rng = cre.extra(&items);
    Success(Thing { kind: ThingKind::List(rng), pos: lex.pos_extend(start) }.push(cre))
  }

  fn read_use_tree(ctx: &mut Ctx) -> Status<ThingId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;
    let first = Self::read_use_sub(ctx!(cre, sin, far, lex, sum, side))?;
    if matches!(cre.get(first).kind, ThingKind::List(..) | ThingKind::Wildcard) {
      return Success(first);
    }

    let mut segments = vec![ first ];

    loop {
      match lex.peek_k(sum)? {
        (WK::Colon2, _) => {
          lex.bump(sum)?;
          let seg = Self::read_use_sub(ctx!(cre, sin, far, lex, sum, side))?;
          segments.push(seg);
          if matches!(cre.get(seg).kind, ThingKind::List(..) | ThingKind::Wildcard) {
            break;
          }
        }
        (WK::As, _) => {
          lex.bump(sum)?;
          let alias = lex.get(sum)?.ident(sum, sin, far)?;
          let last_id = segments.pop().unwrap();
          segments.push(Thing{kind: ThingKind::Alias(last_id, alias), pos: alias.into()}.push(cre));
          break;
        }
        _ => break,
      }
    }

    if segments.len() == 1 {
      Success(segments.pop().unwrap())
    } else {
      let rng = cre.extra(&segments);
      Success(Thing { kind: ThingKind::Path(rng), pos: lex.pos_extend(start) }.push(cre))
    }
  }

  fn read_use_sub(ctx: &mut Ctx) -> Status<ThingId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let ret = match lex.get_k(sum)? {
      (WK::Crate, c) => Thing{kind: ThingKind::Crate, pos: c.into()}.push(cre),
      (WK::Super, c) => Thing{kind: ThingKind::Super, pos: c.into()}.push(cre),
      (WK::SelfS, c) => Thing{kind: ThingKind::Self_, pos: c.into()}.push(cre),
      (WK::Mul, c)   => Thing{kind: ThingKind::Wildcard, pos: c.into()}.push(cre),
      (WK::BraceL, c) => Self::read_use_list(ctx!(cre, sin, far, lex, sum, side), c)?,

      (WK::Word, c) => { let a = c.ident(sum, sin, far)?; Thing{kind: ThingKind::Name(a), pos: a.into()}.push(cre) },

      (_, c) => ret_err!(sum, Message::error(UNKNOWN_USE_SEGMENT, Label::new_pos(c))),
    };

    Success(ret)
  }

  fn sub_mod(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let path = DefPath::Path { base, name: name.sid() }.push(cre);

    let ctn = match lex.get_k(sum)? {
      (WK::Semicolon, _) => None,

      (WK::BraceL, _) => {
        let mut ctn = vec![];
        let defvis = &mut Visibility::Inherited;

        loop {
          if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }
          
          let id = Self::read_item(ctx!(cre, sin, far, lex, sum, side), path, defvis)?;

          ctn.push(id);
        }

        Some(cre.extra(&ctn))
      }

      (_, c) => c.panic_kind2(sum, WK::Semicolon, WK::BraceL)?
    };


    // Post
    Item {
      kind: match ctn {
        None => ItemKind::ModuleUnloaded,
        Some(v) => ItemKind::Module(v),
      },
      name: Some(name),
      pos: lex.pos_extend(start),
      vis,
      path,
    }.push_ok(cre)
  }

  fn sub_itemty(ctx: &mut Ctx, base: DefPathId, vis: Visibility) -> Status<ItemId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let path = DefPath::Path { base, name: name.sid() }.push(cre);
    
    let kind = match start.kind() {
      WK::Struct  => TypeParser::sub_struct(ctx!(cre, sin, far, lex, sum, side), path, name)?,
      WK::Iface   => TypeParser::sub_iface(ctx!(cre, sin, far, lex, sum, side), path)?,
      WK::Trait   => TypeParser::sub_trait(ctx!(cre, sin, far, lex, sum, side), path)?,
      WK::Enum    => TypeParser::sub_enum(ctx!(cre, sin, far, lex, sum, side), path)?,
      WK::Flags   => TypeParser::sub_flags(ctx!(cre, sin, far, lex, sum, side), path)?,
      WK::Variant => TypeParser::sub_variant(ctx!(cre, sin, far, lex, sum, side), path)?,

      _ => unreachable!(),
    };
    
    
    // Post
    Item {
      kind: ItemKind::ItemTy(kind),
      pos: lex.pos_extend(start),
      name: Some(name),
      vis,
      path,
    }.push_ok(cre)
  }

}
