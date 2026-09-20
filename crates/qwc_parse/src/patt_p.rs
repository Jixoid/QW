use qwc_ast::{IdentSave, Patt, PattId};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx};


pub struct PattParser;

impl PattParser {

  pub fn read_patt(ctx: &mut Ctx) -> Result<PattId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let it = match lex.get_k()? {
      (WK::Word, c) => Patt::One(c.ident(sin, far)?),

      (WK::Underscore, _) => Patt::Under,
      (WK::Dot2, _) => Patt::Rest,

      (WK::ParenL, _) => {
        let rng = {
          let mut subs = vec![];
          
          loop {
            if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break }
            
            subs.push(Self::read_patt(ctx!(cre, sin, far, lex, sum))?);
            
            match lex.get_k()? {
              (WK::ParenR, _) => break,
              (WK::Comma, _) => continue,
              
              (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
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
            if lex.peek()?.kind() == WK::BracketR { lex.bump()?; break }
            
            subs.push(Self::read_patt(ctx!(cre, sin, far, lex, sum))?);
            
            match lex.get_k()? {
              (WK::BracketR, _) => break,
              (WK::Comma, _) => continue,
              
              (_, c) => return Err(Message::error(EXPECTED_IDENTIFIER_AFTER, Label::new_pos(c))),
            }
          }
          
          cre.extra(&subs)
        };
          
        Patt::Array(rng)
      }

      (_, c) => return Err(Message::error(UNKNOWN_PATTERN, Label::new_pos(c))),
    };

    Ok(cre.push(it))
  }

}
