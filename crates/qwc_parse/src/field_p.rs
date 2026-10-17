/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Field, FieldId, FieldKind, IdentSave, PushOkApi, Rng, Thing, ThingKind, Visibility};
use qwc_diagnostic::{Label, Message, Status::{self, Success}, msg::*, ret_err};
use qwc_lexer::WK;

use crate::{ctx, ExprParser, MetaParser, WordCheck, Ctx, TypeParser};


pub struct FieldParser;

impl FieldParser {

  // Public
  pub fn read_field(ctx: &mut Ctx, defvis: &mut Visibility) -> Status<FieldId> {
    let (vis, attr) = MetaParser::read_start(ctx, defvis)?;

    let id = match ctx.lex.peek_k(ctx.sum)? {
      (WK::Word, _)    => Self::sub_member (ctx, vis)?,
      (WK::Type, _)    => Self::sub_type   (ctx, vis)?,
      (WK::Fun, _)     => Self::sub_fun    (ctx, vis)?,
      (WK::Init, _)    => Self::sub_init   (ctx, vis)?,
      (WK::Fini, _)    => Self::sub_fini   (ctx, vis)?,
      (WK::Impl, _)    => Self::sub_impl   (ctx, vis)?,


      (_, c) => ret_err!(ctx.sum, Message::error(UNKNOWN_KEYWORD, Label::new_pos(c))),
    };

    if let Some(a) = attr { ctx.cre.attach(id, a); }
    
    Success(id)
  }


  // Sub
  fn sub_member(ctx: &mut Ctx, vis: Visibility) -> Status<FieldId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    lex.get(sum)?.expect_kind(sum, WK::Colon)?;

    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get(sum)?.expect_kind(sum, WK::Semicolon)?;


    // Post
    Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::MemberVar {kind}
    }.push_ok(cre)
  }


  fn sub_type(ctx: &mut Ctx, vis: Visibility) -> Status<FieldId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;
    
    let kind = match lex.peek_k(sum)? {
      (WK::Eq, _) => {
        lex.get(sum)?.expect_kind(sum, WK::Eq)?;
        let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
        lex.get(sum)?.expect_kind(sum, WK::Semicolon)?;
        Some(kind)
      }
      
      (WK::Semicolon, _) => {
        lex.get(sum)?.expect_kind(sum, WK::Semicolon)?;
        None
      }

      (_, c) => c.panic_kind2(sum, WK::Eq, WK::Semicolon)?,
    };

    
    // Post
    Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Type(kind),
    }.push_ok(cre)
  }


  pub fn sub_fun(ctx: &mut Ctx, vis: Visibility) -> Status<FieldId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum, side))?;

    let blok = match lex.peek_k(sum)? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump(sum)?; None },

      (_, c) => c.panic_kind2(sum, WK::BraceL, WK::Semicolon)?
    };


    // Post
    Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Fun{ kind, blok },
    }.push_ok(cre)
  }

  fn sub_init(ctx: &mut Ctx, vis: Visibility) -> Status<FieldId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum, side))?;

    let ils = if lex.peek(sum)?.kind() == WK::Colon {
      lex.bump(sum)?;
      let mut ils = vec![];
      
      loop {
        let start = lex.peek(sum)?;

        let name = if let (WK::Word, c) = lex.peek_k(sum)? { lex.bump(sum)?; c.ident(sum, sin, far)? } else { break };
        
        lex.get(sum)?.expect_kind(sum, WK::ParenL)?;
        
        let v = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

        lex.get(sum)?.expect_kind(sum, WK::ParenR)?;

        
        ils.push(Thing {
          kind: ThingKind::NamedExpr(name, v),
          pos: lex.pos_extend(start),
        }.push(cre));
        
        if lex.peek(sum)?.kind() == WK::Comma { lex.bump(sum)? } else { break }
      }
      
      cre.extra(&ils)
    } else {
      Rng::empty()
    };

    let blok = match lex.peek_k(sum)? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump(sum)?; None },

      (_, c) => c.panic_kind2(sum, WK::BraceL, WK::Semicolon)?
    };


    // Post
    Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Init{ kind, blok, ils }
    }.push_ok(cre)
  }
  
  fn sub_fini(ctx: &mut Ctx, vis: Visibility) -> Status<FieldId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = lex.get(sum)?.ident(sum, sin, far)?;

    let kind = TypeParser::read_fun(ctx!(cre, sin, far, lex, sum, side))?;

    let blok = match lex.peek_k(sum)? {
      (WK::BraceL | WK::Backtick, _) => Some(ExprParser::pre_block(ctx!(cre, sin, far, lex, sum, side))?),
      (WK::Semicolon, _) => { lex.bump(sum)?; None },

      (_, c) => c.panic_kind2(sum, WK::BraceL, WK::Semicolon)?
    };


    // Post
    Field {
      pos: lex.pos_extend(start),
      vis,
      name: Some(name),
      kind: FieldKind::Fini{ kind, blok }
    }.push_ok(cre)
  }


  fn sub_impl(ctx: &mut Ctx, vis: Visibility) -> Status<FieldId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    lex.get(sum)?.expect_kind(sum, WK::Colon)?;

    let trait_ty = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;

    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;

    let ctn = {
      let mut ctn = vec![];
      let defvis = &mut Visibility::Inherited;

      loop {
        if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }
        
        ctn.push( FieldParser::read_field(ctx!(cre, sin, far, lex, sum, side), defvis)? );
      }

      cre.extra(&ctn)
    };


    // Post
    Field {
      pos: lex.pos_extend(start),
      vis,
      name: None,
      kind: FieldKind::ImplIn{ trait_ty, ctn },
    }.push_ok(cre)
  }
  
}

