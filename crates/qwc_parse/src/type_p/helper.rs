/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use qwc_ast::{IdentSave, PushOkApi, Rng, Thing, ThingKind, ThingRng, Type, TypeId, TypeKind, TypeRng};
use qwc_diagnostic::{ret_err, Label, Message, Status::{self, Success}, msg::*};
use qwc_lexer::WK;

use crate::{Ctx, TypeParser, ctx, meta_p::WordCheck};


pub fn read_bases(ctx: &mut Ctx) -> Status<TypeRng> { ctx!(ctx => cre, sin, far, lex, sum, side);
  if lex.peek(sum)?.kind() == WK::Colon {
    lex.bump(sum)?;

    let mut arr = vec![];
    
    loop {
      arr.push( TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))? );

      if lex.peek(sum)?.kind() == WK::Comma { lex.bump(sum)?; continue } else { break }
    }

    Success(cre.extra(&arr))
  }
  else {
    Success(Rng::empty())
  }
}

pub fn read_fun_args(ctx: &mut Ctx) -> Status<(Option<TypeId>, ThingRng)> { ctx!(ctx => cre, sin, far, lex, sum, side);
  lex.get(sum)?.expect_kind(sum, WK::ParenL)?;
  
  let mut args = vec![];
  let mut self_kind = None;

  if lex.peek(sum)?.kind() != WK::ParenR {
    if lex.peek(sum)?.kind() == WK::Amp {
      let amp_word = lex.get(sum)?;
      let ism = if lex.peek(sum)?.kind() == WK::Mut {
        lex.bump(sum)?;
        true
      } else {
        false
      };
      let self_word = lex.get(sum)?;
      if self_word.kind() != WK::SelfS {
        ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(self_word)));
      }
      let self_t = Type {
        pos: lex.pos_extend(self_word),
        kind: TypeKind::SelfT,
      }.push(cre);
      let ref_t = Type {
        pos: lex.pos_extend(amp_word),
        kind: TypeKind::Ref(self_t, ism),
      }.push(cre);
      self_kind = Some(ref_t);
    } else if lex.peek(sum)?.kind() == WK::Mut {
      let mut_word = lex.get(sum)?;
      if lex.peek(sum)?.kind() == WK::SelfS {
        let self_word = lex.get(sum)?;
        let self_t = Type {
          pos: lex.pos_extend(self_word),
          kind: TypeKind::SelfT,
        }.push(cre);
        self_kind = Some(self_t);
      } else {
        ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(mut_word)));
      }
    } else if lex.peek(sum)?.kind() == WK::SelfS {
      let self_word = lex.get(sum)?;
      let self_t = Type {
        pos: lex.pos_extend(self_word),
        kind: TypeKind::SelfT,
      }.push(cre);
      self_kind = Some(self_t);
    }

    if self_kind.is_some() {
      match lex.peek_k(sum)? {
        (WK::Comma, _) => { lex.bump(sum)?; }
        (WK::ParenR, _) => {
          lex.bump(sum)?;
          return Success((self_kind, cre.extra(&args)));
        }
        (_, c) => c.panic_kind2(sum, WK::Comma, WK::ParenR)?
      }
    }
  }

  loop {
    if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break; }
    
    let mut names = vec![];
    loop {
      let name = match lex.get_k(sum)? {
        (WK::Word, c) => c.ident(sum, sin, far)?,
        (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER, Label::new_pos(c))),
      };
      names.push(name);

      match lex.get_k(sum)? {
        (WK::Comma, _) => continue,
        (WK::Colon, _) => break,

        (_, c) => c.panic_kind2(sum, WK::Colon, WK::Comma)?
      }
    }

    let kind = TypeParser::read_type(ctx!(cre, sin, far, lex, sum, side))?;
    
    for x in names {
      args.push(Thing {
        kind: ThingKind::NamedType(x, kind),
        pos: x.into(),
      }.push(cre));
    }

    match lex.get_k(sum)? {
      (WK::ParenR, _) => break,
      (WK::Comma, _) => continue,

      (_, c) => c.panic_kind2(sum, WK::Comma, WK::ParenL)?
    }
  }

  Success((self_kind, cre.extra(&args)))
}
