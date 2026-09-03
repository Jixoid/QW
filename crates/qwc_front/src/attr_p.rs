use qwc_ast::{Attribute, IdentSave};
use qwc_diagnostic::Message;
use qwc_lexer::WK;
use thin_vec::ThinVec;

use crate::{WordCheck, ctx, front::Ctx, ExprParser};


pub struct AttrParser;

impl AttrParser {

  pub fn read_attr(ctx: &mut Ctx) -> Result<Option<Vec<Attribute>>, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let mut attrs = vec![];

    loop {
      if lex.peek()?.kind() == WK::Attribute {
        lex.bump()?;

        loop {
          if lex.peek()?.kind() == WK::SquareBracketEnd { lex.bump()?; break }
          
          attrs.push(Self::read_attr_sub(ctx!(cre, sin, far, lex, sum))?);

          match lex.get_k()? {
            (WK::Comma, _) => continue,
            (WK::SquareBracketEnd, _) => break,
          
            (_, c) => c.panic_kind2(WK::Comma, WK::SquareBracketEnd)?
          }
        }
      } else {
        break;
      }
    }

    Ok(if attrs.is_empty() { None } else { Some(attrs) })
  }


  fn read_attr_sub(ctx: &mut Ctx) -> Result<Attribute, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let key = lex.get()?;
    
    let attr = match lex.peek()?.kind() {
      WK::Colon => { // A:B
        lex.bump()?;
        let val = lex.get()?;

        Attribute::Bin(key.ident(sin, far)?, val.ident(sin, far)?)
      }

      WK::ParenBeg => { // A(B,C)
        lex.bump()?;
        let mut attrs = ThinVec::new();

        loop {
          if lex.peek()?.kind() == WK::ParenEnd { lex.bump()?; break }
          
          attrs.push(Self::read_attr_sub(ctx!(cre, sin, far, lex, sum))?);

          match lex.get_k()? {
            (WK::Comma, _) => continue,
            (WK::ParenEnd, _) => break,
            
            (_, c) => c.panic_kind2(WK::Comma, WK::ParenEnd)?
          }
        }

        Attribute::List(key.ident(sin, far)?, attrs)
      }

      WK::Assign => { // A = $expr
        lex.bump()?;
        let ex = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;

        Attribute::Set(key.ident(sin, far)?, ex)
      }

      _ => Attribute::One(key.ident(sin, far)?)
    };

    Ok(attr)
  }

}
