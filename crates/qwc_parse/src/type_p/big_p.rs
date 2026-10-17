/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{self as ast, DefPathId, FieldKind, Ident, IdentSave, Item, PushOkApi, Thing, ThingKind, Type, TypeId, TypeKind, Visibility};
use qwc_diagnostic::{ret_err, Label, Message, Status::{self, Success}, msg::*};
use qwc_lexer::WK;

use crate::{Ctx, FieldParser, TypeParser, WordCheck, ctx, ExprParser, type_p::helper};


impl TypeParser {

  // Big
  pub fn sub_struct(ctx: &mut Ctx, path: DefPathId, name: Ident) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;

    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;

    let (rng, impin, hideimp) = {
      let mut ctn = vec![];
      let mut impin = vec![];
      let mut hideimp = vec![];

      loop {
        if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break }

        let id = FieldParser::read_field(ctx!(cre, sin, far, lex, sum, side), &mut Visibility::Inherited)?;
        let it = cre.get(id);
        
        match it.kind {
          FieldKind::MemberVar {kind} => ctn.push(Thing{kind: ThingKind::NamedType(it.name.unwrap(), kind), pos: it.name.unwrap().into()}.push(cre)),
          
          FieldKind::ImplIn {trait_ty, ctn} => impin.push((trait_ty, ctn, it.pos, it.vis)),

          FieldKind::Fun{..} | FieldKind::Init{..} | FieldKind::Fini{..} => hideimp.push(id),

          FieldKind::Type(..) => panic!("unexpected kind"),
        }
      }

      (cre.extra(&ctn), impin.into_boxed_slice(), hideimp.into_boxed_slice())
    };

    let type_ty = Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Struct { vars: rng, path },
    }.push(ctx.cre);

    let name_ty = Type {
      pos: name.into(),
      kind: TypeKind::Nick(name),
    }.push(ctx.cre);
    
    for (trait_ty, ctn, pos, vis) in impin {
      let sideimp = Item {
        kind: ast::ItemKind::Impl { type_ty: name_ty, trait_ty: Some(trait_ty), ctn },
        name: None,
        pos,
        vis,
        path,
      }.push(ctx.cre);

      ctx.side.push(sideimp);
    }

    if !hideimp.is_empty() {
      let hideimp = Item {
        kind: ast::ItemKind::Impl { type_ty: name_ty, trait_ty: None, ctn: ctx.cre.extra(&hideimp) },
        name: None,
        pos: lex.pos_extend(start),
        vis: Visibility::Public,
        path,
      }.push(ctx.cre);

      ctx.side.push(hideimp);
    }

    Success(type_ty)
  }

  pub fn sub_enum(ctx: &mut Ctx, path: DefPathId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;
    
    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;

    let vals = {
      let mut vals = vec![];
      
      loop {
        let name = match lex.get_k(sum)? {
          (WK::Word, c) => c.ident(sum, sin, far)?,
          (WK::BraceR, _) => break,
          
          (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };
        
        let item = if lex.peek(sum)?.kind() == WK::Eq {
          lex.bump(sum)?;
          let val = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
          
          Thing {
            kind: ThingKind::NamedExpr(name, val),
            pos: name.into(),
          }
        } else {
          Thing {
            kind: ThingKind::Name(name),
            pos: name.into(),
          }
        };

        vals.push(item.push(cre));

        match lex.get_k(sum)? {
          (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break },

          (WK::BraceR, _) => break,
          
          (_, c) => c.panic_kind2(sum, WK::Comma, WK::BraceR)?,
        }
      }

      cre.extra(&vals)
    };

    
    // Post
    Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Enum { fields: vals, path },
    }.push_ok(cre)
  }

  pub fn sub_flags(ctx: &mut Ctx, path: DefPathId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;
    
    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;

    let vals = {
      let mut vals = vec![];
      
      loop {
        let name = match lex.get_k(sum)? {
          (WK::Word, c) => c.ident(sum, sin, far)?,
          (WK::BraceR, _) => break,
          
          (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };
        
        let item = if lex.peek(sum)?.kind() == WK::Eq {
          lex.bump(sum)?;
          let val = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;
          
          Thing {
            kind: ThingKind::NamedExpr(name, val),
            pos: name.into(),
          }
        } else {
          Thing {
            kind: ThingKind::Name(name),
            pos: name.into(),
          }
        };

        vals.push(item.push(cre));

        match lex.get_k(sum)? {
          (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break },

          (WK::BraceR, _) => break,
          
          (_, c) => c.panic_kind2(sum, WK::Comma, WK::BraceR)?,
        }
      }

      cre.extra(&vals)
    };

    
    // Post
    Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Flags { fields: vals, path },
    }.push_ok(cre)
  }

  pub fn sub_variant(ctx: &mut Ctx, path: DefPathId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;
    
    lex.get(sum)?.expect_kind(sum, WK::BraceL)?;

    let vals = {
      let mut vals = vec![];

      loop {
        let name = match lex.get_k(sum)? {
          (WK::Word, c) => c.ident(sum, sin, far)?,
          (WK::BraceR, _) => break,
          
          (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
        };

        let item = if lex.peek(sum)?.kind() == WK::ParenL {
          let payload_type = TypeParser::pre_tuple(ctx!(cre, sin, far, lex, sum, side))?;
          
          Thing {
            kind: ThingKind::NamedType(name, payload_type),
            pos: name.into(),
          }
        } else {
          Thing {
            kind: ThingKind::Name(name),
            pos: name.into(),
          }
        };

        vals.push(item.push(cre));

        match lex.get_k(sum)? {
          (WK::Comma, _) => if lex.peek(sum)?.kind() == WK::BraceR { lex.bump(sum)?; break },
          (WK::BraceR, _) => break,

          (_, c) => c.panic_kind2(sum, WK::Comma, WK::BraceR)?,
        }
      }

      cre.extra(&vals)
    };


    // Post
    Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Variant { fields: vals, path },
    }.push_ok(cre)
  }

  pub fn sub_iface(ctx: &mut Ctx, path: DefPathId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;

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

    
    // Post
    Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Iface { fields: rng, path },
    }.push_ok(cre)
  }

  pub fn sub_trait(ctx: &mut Ctx, path: DefPathId) -> Status<TypeId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let start = lex.peek(sum)?;
    
    let reqs = helper::read_bases(ctx!(cre, sin, far, lex, sum, side))?;

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

    
    // Post
    Type {
      pos: lex.pos_extend(start),
      kind: TypeKind::Trait { fields: rng, reqs, path },
    }.push_ok(cre)
  }

}
