use qwc_ast::{IdentSave, Patt, PattId};
use qwc_diagnostic::Message;
use qwc_lexer::WK;

use crate::{parse::Ctx, ctx};


pub struct PattParser;

impl PattParser {

  pub fn read_patt(ctx: &mut Ctx) -> Result<PattId, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let it = match lex.get_k()? {
      (WK::Word, c) => Patt::One(c.ident(sin, far)?),

      (WK::Underscore, _) => Patt::Under,
      (WK::Dot2, _) => Patt::Rest,

      (WK::ParenBeg, _) => {
        let rng = {
          let mut subs = vec![];
          
          loop {
            if lex.peek()?.kind() == WK::ParenEnd { lex.bump()?; break }
            
            subs.push(Self::read_patt(ctx!(cre, sin, far, lex, sum))?);
            
            match lex.get_k()? {
              (WK::ParenEnd, _) => break,
              (WK::Comma, _) => continue,
              
              (_, c) => return Err(Message::error(c, "expected ',' or ')' in tuple pattern", &[])),
            }
          }
          
          cre.extra(&subs)
        };
          
        Patt::Tuple(rng)
      }

      (WK::SquareBracketBeg, _) => {
        let rng = {
          let mut subs = vec![];
          
          loop {
            if lex.peek()?.kind() == WK::SquareBracketEnd { lex.bump()?; break }
            
            subs.push(Self::read_patt(ctx!(cre, sin, far, lex, sum))?);
            
            match lex.get_k()? {
              (WK::SquareBracketEnd, _) => break,
              (WK::Comma, _) => continue,
              
              (_, c) => return Err(Message::error(c, "expected ',' or ']' in array pattern", &[])),
            }
          }
          
          cre.extra(&subs)
        };
          
        Patt::Array(rng)
      }

      (_, c) => return Err(Message::error(c, "unknown pattern", &[])),
    };

    Ok(cre.push(it))
  }

}
