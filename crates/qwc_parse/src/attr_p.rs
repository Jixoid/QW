/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{AttrKind, Attribute, IdentSave};
use qwc_diagnostic::Status::{self, Success};
use qwc_lexer::WK;
use thin_vec::ThinVec;

use crate::{WordCheck, ctx, parse::Ctx, ExprParser};


pub struct AttrParser;

impl AttrParser {

  pub fn read_attr(ctx: &mut Ctx) -> Status<Option<Vec<Attribute>>> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let mut attrs = vec![];

    loop {
      if lex.peek(sum)?.kind() == WK::BangAttr {
        lex.bump(sum)?;

        loop {
          if lex.peek(sum)?.kind() == WK::BracketR { lex.bump(sum)?; break }
          
          attrs.push(Self::read_attr_sub(ctx!(cre, sin, far, lex, sum, side))?);

          match lex.get_k(sum)? {
            (WK::Comma, _) => continue,
            (WK::BracketR, _) => break,
          
            (_, c) => c.panic_kind2(sum, WK::Comma, WK::BracketR)?
          }
        }
      } else {
        break;
      }
    }

    Success(if attrs.is_empty() { None } else { Some(attrs) })
  }


  fn read_attr_sub(ctx: &mut Ctx) -> Status<Attribute> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let key = lex.get(sum)?;
    
    let attr = match lex.peek(sum)?.kind() {
      WK::Colon => { // A:B
        lex.bump(sum)?;
        let val = lex.get(sum)?;

        Attribute{ident: key.ident(sum, sin, far)?, kind: AttrKind::Bin(val.ident(sum, sin, far)?)}
      }

      WK::ParenL => { // A(B,C)
        lex.bump(sum)?;
        let mut attrs = ThinVec::new();

        loop {
          if lex.peek(sum)?.kind() == WK::ParenR { lex.bump(sum)?; break }
          
          attrs.push(Self::read_attr_sub(ctx!(cre, sin, far, lex, sum, side))?);

          match lex.get_k(sum)? {
            (WK::Comma, _) => continue,
            (WK::ParenR, _) => break,
            
            (_, c) => c.panic_kind2(sum, WK::Comma, WK::ParenR)?
          }
        }

        Attribute{ident: key.ident(sum, sin, far)?, kind: AttrKind::List(attrs)}
      }

      WK::Eq => { // A = $expr
        lex.bump(sum)?;
        let expr = ExprParser::read_expr(ctx!(cre, sin, far, lex, sum, side))?;

        Attribute{ident: key.ident(sum, sin, far)?, kind: AttrKind::Set(expr)}
      }

      _ => Attribute{ident: key.ident(sum, sin, far)?, kind: AttrKind::One()}
    };

    Success(attr)
  }

}
