use crate::{ast::attr::Attribute, lexer::WordKind};
use super::front::ParserContext;

pub struct AttrParser {}

impl AttrParser {

  pub fn read_attributes<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Vec<Attribute<'a>> {
    let mut attrs = Vec::new();
    loop {
      let t = ctx.lex.lex();
      if t.kind == WordKind::DoubleSquareBracketBeg {
        // Parse attributes inside [[ ]]
        loop {
          let t2 = ctx.lex.lex();
          if t2.kind == WordKind::DoubleSquareBracketEnd {
            break;
          }
          if t2.kind == WordKind::EOF {
            break;
          }

          if t2.kind != WordKind::Word {
            // expected a word for attribute key
            continue;
          }

          let key = t2;
          let mut val = None;

          // Check for colon
          let next_t = ctx.lex.lex();
          if next_t.kind == WordKind::Colon {
            let v = ctx.lex.lex();
            if v.kind == WordKind::Word || v.kind == WordKind::String {
              val = Some(v);
            }
            // Skip the next comma if present
            let t3 = ctx.lex.lex();
            if t3.kind != WordKind::Comma && t3.kind != WordKind::DoubleSquareBracketEnd {
              ctx.lex.store(t3);
            } else if t3.kind == WordKind::DoubleSquareBracketEnd {
              attrs.push(Attribute { key, val });
              break;
            }
          } else if next_t.kind == WordKind::Comma {
            // just a key
          } else if next_t.kind == WordKind::DoubleSquareBracketEnd {
            attrs.push(Attribute { key, val });
            break;
          } else {
            ctx.lex.store(next_t);
          }
          
          attrs.push(Attribute { key, val });
        }
      } else {
        ctx.lex.store(t);
        break;
      }
    }
    attrs
  }

  pub fn attach_attributes<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>, id: crate::control::identy::IdentyId, attrs: Vec<Attribute<'a>>) {
    if !attrs.is_empty() {
      ctx.mol.map_attr.insert(id, attrs);
    }
  }

}
