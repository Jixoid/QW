use qwc_ast::{AttrKind, Attribute, IdentSave};
use qwc_diagnostic::Message;
use qwc_lexer::WK;
use thin_vec::ThinVec;

use crate::{WordCheck, ctx, parse::Ctx, ExprParser};


pub struct AttrParser;

impl AttrParser {

  pub fn read_attr(ctx: &mut Ctx) -> Result<Option<Vec<Attribute>>, Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let mut attrs = vec![];

    loop {
      if lex.peek()?.kind() == WK::BangAttr {
        lex.bump()?;

        loop {
          if lex.peek()?.kind() == WK::BracketR { lex.bump()?; break }
          
          attrs.push(Self::read_attr_sub(ctx!(cre, sin, far, lex, sum))?);

          match lex.get_k()? {
            (WK::Comma, _) => continue,
            (WK::BracketR, _) => break,
          
            (_, c) => c.panic_kind2(WK::Comma, WK::BracketR)?
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

        Attribute{ident: key.ident(sin, far)?, kind: AttrKind::Bin(val.ident(sin, far)?)}
      }

      WK::ParenL => { // A(B,C)
        lex.bump()?;
        let mut attrs = ThinVec::new();

        loop {
          if lex.peek()?.kind() == WK::ParenR { lex.bump()?; break }
          
          attrs.push(Self::read_attr_sub(ctx!(cre, sin, far, lex, sum))?);

          match lex.get_k()? {
            (WK::Comma, _) => continue,
            (WK::ParenR, _) => break,
            
            (_, c) => c.panic_kind2(WK::Comma, WK::ParenR)?
          }
        }

        Attribute{ident: key.ident(sin, far)?, kind: AttrKind::List(attrs)}
      }

      WK::Eq => { // A = $expr
        lex.bump()?;
        let expr = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum))?;

        Attribute{ident: key.ident(sin, far)?, kind: AttrKind::Set(expr)}
      }

      _ => Attribute{ident: key.ident(sin, far)?, kind: AttrKind::One()}
    };

    Ok(attr)
  }

}
