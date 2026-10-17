/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{IdentSave, Patt, PattId, PushOkApi};
use qwc_diagnostic::{Label, Message, Status, msg::*, ret_err};
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx};


pub struct PattParser;

impl PattParser {

  pub fn read_patt(ctx: &mut Ctx) -> Status<PattId> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let it = match lex.get_k(sum)? {
      (WK::Word, c) => Patt::One(c.ident(sum, sin, far)?),

      (WK::Underscore, _) => Patt::Under,
      (WK::Dot2, _) => Patt::Rest,

      (WK::ParenL, _) => {
        let rng = {
          let mut subs = vec![];
          
          loop {
            if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break }
            
            subs.push(Self::read_patt(ctx!(cre, sin, far, lex, sum, side))?);
            
            match lex.get_k(sum)? {
              (WK::ParenR, _) => break,
              (WK::Comma, _) => continue,
              
              (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
            }
          }
          
          cre.extra(&subs)
        };
          
        Patt::Tuple(rng)
      }

      (WK::BracketL, _) => {
        let rng = {
          let mut subs = vec![];
          
          loop {
            if lex.peek(sum)?.kind() == WK::BracketR { lex.bump(sum)?; break }
            
            subs.push(Self::read_patt(ctx!(cre, sin, far, lex, sum, side))?);
            
            match lex.get_k(sum)? {
              (WK::BracketR, _) => break,
              (WK::Comma, _) => continue,
              
              (_, c) => ret_err!(sum, Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
            }
          }
          
          cre.extra(&subs)
        };
          
        Patt::Array(rng)
      }

      (_, c) => ret_err!(sum, Message::error(UNKNOWN_PATTERN, Label::new_pos(c))),
    };

    it.push_ok(cre)
  }

}
