use crate::{ast::{AnyId, AstId, Attr, AttrId, AttrVari}, diagnostic::Message, front::expr_p::ExprParser, lexer::WK};
use super::front::ParserContext;


pub struct AttrParser {}

impl AttrParser {

  pub fn read_attr(ctx: &mut ParserContext) -> Result<Vec<AttrId>, Message> {
    let mut attrs = Vec::new();

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WK::Attribute {

        loop {
          let t = ctx.lex.get()?;
          if t.kind == WK::SquareBracketEnd { break; } else { ctx.lex.store(t); }
          
          attrs.push(Self::read_attr_sub(ctx)?);

          let t = ctx.lex.get()?;
          if t.kind == WK::Comma { continue; }
          else if t.kind == WK::SquareBracketEnd { break; }
          else {
            t.expect_kind2(WK::Comma, WK::SquareBracketEnd)?;
          }
        }

      } else {
        ctx.lex.store(t);
        break;
      }
    }

    Ok(attrs)
  }


  fn read_attr_sub(ctx: &mut ParserContext) -> Result<AttrId, Message> {
    let key = ctx.lex.get()?;
    
    let _c = ctx.lex.get()?;

    match _c.kind {
      WK::Colon => { // A:B
        let val = ctx.lex.get()?;

        let this = Attr{vari: AttrVari::Bin(key.save(), val.save())};

        Ok(ctx.cre.new_attr(this))
      }

      WK::ParenBeg => { // A(B,C)
        let mut attrs = vec![];

        loop {
          let t = ctx.lex.get()?;
          if t.kind == WK::ParenEnd { break; } else { ctx.lex.store(t); }
          
          attrs.push(Self::read_attr_sub(ctx)?.to_any());

          let t = ctx.lex.get()?;
          if t.kind == WK::Comma { continue; }
          else if t.kind == WK::ParenEnd { break; }
          else {
            t.expect_kind2(WK::Comma, WK::ParenEnd)?;
          }
        }

        let rng = ctx.cre.new_extra(attrs);

        let this = Attr{vari: AttrVari::List(key.save(), rng)};

        Ok(ctx.cre.new_attr(this))
      }

      WK::Assign => { // A = $expr
        let ex = ExprParser::read_expr(ctx, 0)?;

        let this = Attr{vari: AttrVari::Set(key.save(), ex)};

        Ok(ctx.cre.new_attr(this))
      }

      _ => {
        ctx.lex.store(_c);

        let this = Attr{vari: AttrVari::One(key.save())};

        return Ok(ctx.cre.new_attr(this));
      }
    
    }
  }


  pub fn attach_attr<T>(ctx: &mut ParserContext, id: AstId<T>, attrs: Vec<AttrId>) {
    if !attrs.is_empty() {
      ctx.cre.map_attr.insert(AnyId::new_from(id), attrs);
    }
  }

}
