use crate::{ast::{Patt, PattId}, diagnostic::Message, front::ParserContext, lexer::WK};


pub struct PattParser {}

impl PattParser {

  pub fn read_patt(ctx: &mut ParserContext) -> Result<PattId, Message> {
    let _c = ctx.lex.get()?;

    match _c.kind {
      WK::Word => {
        _c.expect_word(ctx.far)?;

        let this = Patt::One(_c.save());
        Ok(ctx.cre.new_patt(this))
      }

      WK::Underscore => { // `_`
        let this = Patt::Under{};
        Ok(ctx.cre.new_patt(this))
      }

      WK::Dot2 => { // `..`
        let this = Patt::Rest{};
        Ok(ctx.cre.new_patt(this))
      }

      WK::ParenBeg => { // `(`
        let mut subs = vec![];

        loop {
          let _c = ctx.lex.get()?;

          if _c.kind == WK::ParenEnd { break; }
          else { ctx.lex.store(_c);}
          
          subs.push(Self::read_patt(ctx)?.to_any());
          
          let next_c = ctx.lex.get()?;
          if next_c.kind == WK::ParenEnd { break; }
          else if next_c.kind == WK::Comma { continue; }
          else {
            return Err(Message::error(next_c.save(), "expected ',' or ')' in tuple pattern", vec![]));
          }
        }

        let rng = ctx.cre.new_extra(subs);
        
        let this = Patt::Tuple(rng);
        Ok(ctx.cre.new_patt(this))
      }

      WK::SquareBracketBeg => { // `[`
        let mut subs = vec![];

        loop {
          let _c = ctx.lex.get()?;

          if _c.kind == WK::SquareBracketEnd { break; }
          else { ctx.lex.store(_c);}
          
          subs.push(Self::read_patt(ctx)?.to_any());
          
          let next_c = ctx.lex.get()?;
          if next_c.kind == WK::SquareBracketEnd { break; }
          else if next_c.kind == WK::Comma { continue; }
          else {
            return Err(Message::error(next_c.save(), "expected ',' or ']' in array pattern", vec![]));
          }
        }

        let rng = ctx.cre.new_extra(subs);
        
        let this = Patt::Array(rng);
        Ok(ctx.cre.new_patt(this))
      }

      _ => Err(Message::error(_c.save(), "unknown pattern", vec![])),
    }
  }

}
