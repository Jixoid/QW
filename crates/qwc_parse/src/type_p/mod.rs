/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{AnyRng, IdentSave, PushOkApi, Type, TypeId, TypeKind};
use qwc_diagnostic::Status::{self, Success};
use qwc_lexer::WK;

use crate::{AttrParser, ExprParser, WordCheck, ctx, Ctx};

mod helper;
mod big_p;



pub struct TypeParser;

impl TypeParser {

  // Public
  pub fn read_type(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let attrs = AttrParser::read_attr(ctx!(cre, sin, far, lex, sum, side))?;
    
    lex.split_token(sum)?;

    let mut lhs = match lex.peek(sum)?.kind() {
      WK::Question => Self::pre_option(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Bang     => Self::pre_fail(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Dot2     => Self::pre_range(ctx!(cre, sin, far, lex, sum, side))?,

      WK::Amp   => Self::pre_ref(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Caret => Self::pre_ptr(ctx!(cre, sin, far, lex, sum, side))?,
      
      WK::BracketL => Self::pre_slice_array_vector(ctx!(cre, sin, far, lex, sum, side))?,

      WK::ParenL => Self::pre_tuple(ctx!(cre, sin, far, lex, sum, side))?,

      WK::Fun  => Self::pre_fun(ctx!(cre, sin, far, lex, sum, side), true)?,
      WK::Task => Self::pre_task(ctx!(cre, sin, far, lex, sum, side), true)?,

      WK::SelfB => Self::pre_self(ctx!(cre, sin, far, lex, sum, side))?,
      WK::Type  => Self::pre_type(ctx!(cre, sin, far, lex, sum, side))?,

      _ => Self::pre_nick(ctx!(cre, sin, far, lex, sum, side))?
    };

    loop {
      lhs = match lex.peek(sum)?.kind() {
        WK::Colon2 => Self::post_scope(ctx!(cre, sin, far, lex, sum, side), lhs)?,
        
        WK::Lt => Self::post_specialize(ctx!(cre, sin, far, lex, sum, side), lhs)?,

        _ => break
      };
    }

    if let Some(a) = attrs { cre.attach(lhs, a); }

    Success(lhs)
  }

  pub fn read_fun(ctx: &mut Ctx) -> Status<TypeId> { Self::pre_fun(ctx, false) }
  pub fn read_task(ctx: &mut Ctx) -> Status<TypeId> { Self::pre_task(ctx, false) }


  // Ex
  fn post_scope(ctx: &mut Ctx, lhs: TypeId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let rng = {
      let sub = Self::pre_nick(ctx!(cre, sin, far, lex, sum, side))?;
      
      let mut ctn = vec![lhs, sub];

      loop {
        if lex.peek(sum)?.kind() == WK::Colon2 {
          lex.bump(sum)?;

          ctn.push(Self::pre_nick(ctx!(cre, sin, far, lex, sum, side))?);
        } else {
          break
        }
      }
      
      cre.extra(&ctn)
    };
      

    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Path(rng)
    }.push_ok(cre)
  }

  fn post_specialize(ctx: &mut Ctx, lhs: TypeId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let args = if lex.peek(sum)?.kind() == WK::Gt {
      lex.bump(sum)?;
      AnyRng::empty()
    } else {
      let mut args = vec![];

      loop {
        let arg = match lex.peek_k(sum)? {
          (WK::String | WK::Number | WK::BraceL, _) => ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?.to_any(),
          
          _ => TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?.to_any(),
        };
        
        args.push(arg);

        lex.split_token(sum)?;

        match lex.get_k(sum)? {
          (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::Gt { lex.bump(sum)?; break },
          
          (WK::Gt, _) => break,

          (_, c) => c.panic_kind2(sum, WK::Comma, WK::Gt)?,
        }
      }

      cre.extra_any(&args)
    };

    
    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Spec{ base: lhs, args }
    }.push_ok(cre)
  }



  // Sub
  fn pre_self(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::SelfT,
    }.push_ok(cre)
  }

  fn pre_type(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Type,
    }.push_ok(cre)
  }
  

  fn pre_option(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Option(sub)
    }.push_ok(cre)
  }

  fn pre_fail(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fail(sub)
    }.push_ok(cre)
  }

  fn pre_range(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Range(sub)
    }.push_ok(cre)
  }

  fn pre_ref(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let ism = if lex.peek(sum)?.kind() == WK::Mut { lex.bump(sum)?; true } else { false };

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Ref(sub, ism)
    }.push_ok(cre)
  }

  fn pre_ptr(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let ism = if lex.peek(sum)?.kind() == WK::Mut { lex.bump(sum)?; true } else { false };

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Ptr(sub, ism)
    }.push_ok(cre)
  }


  fn pre_fun(ctx: &mut Ctx, is_sub: bool) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = if is_sub { lex.get(sum)? } else { lex.peek(sum)? };

    let (self_kind, args) = helper::read_fun_args(ctx!(cre, sin, far, lex, sum, side))?;

    let ret = if lex.peek(sum)?.kind() == WK::ArrowRight {
      lex.bump(sum)?;
      Some(Self::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fun{ self_kind, args, ret, attr: 0 }
    }.push_ok(cre)
  }

  fn pre_task(ctx: &mut Ctx, is_sub: bool) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = if is_sub { lex.get(sum)? } else { lex.peek(sum)? };

    let (self_kind, args) = helper::read_fun_args(ctx!(cre, sin, far, lex, sum, side))?;

    let ret = if lex.peek(sum)?.kind() == WK::ArrowRight {
      lex.bump(sum)?;
      Some(Self::read_type(ctx!(cre, sin, far, lex, sum, side))?)
    } else {
      None
    };


    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Fun{ self_kind, args, ret, attr: 0 }
    }.push_ok(cre)
  }


  fn pre_slice_array_vector(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    let sub = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;

    let kind = match lex.get_k(sum)? {
      (WK::BracketR, _) => TypeKind::Slice(sub),

      (WK::Semicolon, _) => {
        let ext = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

        lex.get(sum)?.expect_kind(sum, WK::BracketR)?;

        TypeKind::Array(sub, ext)
      }

      (WK::Mul, _) => {
        let ext = if lex.peek(sum)?.kind() == WK::BracketR { None } else { Some(ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?) };

        lex.get(sum)?.expect_kind(sum, WK::BracketR)?;

        if let Some(ext) = ext {
          TypeKind::Vector(sub, ext)
        } else {
          TypeKind::VScale(sub)
        }
      }

      (_, c) => c.panic_kind3(sum, WK::BracketR, WK::Semicolon, WK::Mul)?
    };
    

    // Post
    Type{
      pos: lex.pos_extend(start),
      kind
    }.push_ok(cre)
  }

  fn pre_tuple(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;

    // Unit
    if lex.peek(sum)?.kind() == WK::ParenR {
      lex.bump(sum)?;

      // Post
      return Type{
        pos: lex.pos_extend(start),
        kind: TypeKind::Unit,
      }.push_ok(cre)
    }

    let kind = Self::read_type(ctx!(cre, sin, far, lex, sum, side))?;

    match lex.get_k(sum)? {
      (WK::ParenR, _) => Success(kind), // (X)

      (WK::Comma, _) => { // (X, ...)
        let mut vals = vec![ kind ];
        let mut is_tuple = false;
        
        loop {
          if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break }
          
          vals.push(Self::read_type(ctx!(cre, sin, far, lex, sum, side))?);
          
          match lex.get_k(sum)? {
            (WK::ParenR, _) => break,
            (WK::Comma, _) => { is_tuple = true; continue; }
            
            (_, c) => c.panic_kind2(sum, WK::Comma, WK::ParenR)?
          }
        }
        
        if is_tuple || vals.len() != 1 {
          let rng = cre.extra(&vals);

          Type{
            pos: lex.pos_extend(start),
            kind: TypeKind::TupleAnon(rng)
          }.push_ok(cre)
        } else {
          Success(vals[0])
        }
      }

      (_, c) => c.panic_kind2(sum, WK::ParenR, WK::Comma)?
    }
  }


  fn pre_nick(ctx: &mut Ctx) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.get(sum)?;
    let name = start.ident(sum, sin, far)?;

    // Post
    Type{
      pos: lex.pos_extend(start),
      kind: TypeKind::Nick(name)
    }.push_ok(cre)
  }

}
