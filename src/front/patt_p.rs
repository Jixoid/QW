use crate::{ast::{Patt, PattId}, diagnostic::Message, front::ParserContext, lexer::WordKind};


pub struct PattParser {}

impl PattParser {

  pub fn read_patt<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<PattId, Message<'a>> {
    let _c = ctx.lex.get()?;

    match _c.kind {
      WordKind::Word => {
        _c.expect_word()?;

        let this = Patt::One(_c);
        Ok(ctx.cre.new_patt(this))
      }

      WordKind::Underscore => { // `_`
        let this = Patt::Wildcard{};
        Ok(ctx.cre.new_patt(this))
      }

      WordKind::Dot2 => { // `..`
        let this = Patt::Rest{};
        Ok(ctx.cre.new_patt(this))
      }

      WordKind::ParenBeg => { // `(`
        let mut subs = vec![];

        loop {
          let _c = ctx.lex.get()?;

          if _c.kind == WordKind::ParenEnd { break; }
          else { ctx.lex.store(_c);}
          
          subs.push(Self::read_patt(ctx)?.to_any());
          
          let next_c = ctx.lex.get()?;
          if next_c.kind == WordKind::ParenEnd { break; }
          else if next_c.kind == WordKind::Comma { continue; }
          else {
            return Err(Message::error(next_c, "expected ',' or ')' in tuple pattern".to_string(), vec![]));
          }
        }

        let rng = ctx.cre.new_extra(subs);
        
        let this = Patt::Tuple(rng);
        Ok(ctx.cre.new_patt(this))
      }

      WordKind::SquareBracketBeg => { // `[`
        let mut subs = vec![];

        loop {
          let _c = ctx.lex.get()?;

          if _c.kind == WordKind::SquareBracketEnd { break; }
          else { ctx.lex.store(_c);}
          
          subs.push(Self::read_patt(ctx)?.to_any());
          
          let next_c = ctx.lex.get()?;
          if next_c.kind == WordKind::SquareBracketEnd { break; }
          else if next_c.kind == WordKind::Comma { continue; }
          else {
            return Err(Message::error(next_c, "expected ',' or ']' in array pattern".to_string(), vec![]));
          }
        }

        let rng = ctx.cre.new_extra(subs);
        
        let this = Patt::Array(rng);
        Ok(ctx.cre.new_patt(this))
      }

      _ => Err(Message::error(_c, "unknown pattern".to_string(), vec![])),
    }
  }

}
