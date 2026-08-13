use crate::{ast::{AnyId, AstId, Attr, AttrId, AttrVari}, diagnostic::Message, front::expr_p::ExprParser, lexer::WordKind};
use super::front::ParserContext;


pub struct AttrParser {}

impl AttrParser {

  pub fn read_attr<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<Vec<AttrId>, Message<'a>> {
    let mut attrs = Vec::new();

    loop {
      let t = ctx.lex.get()?;
      if t.kind == WordKind::Attribute {

        loop {
          let t = ctx.lex.get()?;
          if t.kind == WordKind::SquareBracketEnd { break; } else { ctx.lex.store(t); }
          
          attrs.push(Self::read_attr_sub(ctx)?);

          let t = ctx.lex.get()?;
          if t.kind == WordKind::Comma { continue; }
          else if t.kind == WordKind::SquareBracketEnd { break; }
          else {
            t.expect_equal_str2(",", "]")?;
          }
        }

      } else {
        ctx.lex.store(t);
        break;
      }
    }

    Ok(attrs)
  }


  fn read_attr_sub<'a, 'ctx, 'd>(ctx: &mut ParserContext<'a, 'ctx, 'd>) -> Result<AttrId, Message<'a>> {
    let key = ctx.lex.get()?;
    
    let _c = ctx.lex.get()?;

    match _c.kind {
      WordKind::Colon => { // A:B
        let val = ctx.lex.get()?;

        let this = Attr{vari: AttrVari::Bin(key, val)};

        Ok(ctx.cre.new_attr(this))
      }

      WordKind::ParenBeg => { // A(B,C)
        let mut attrs = Vec::new();

        loop {
          let t = ctx.lex.get()?;
          if t.kind == WordKind::ParenEnd { break; } else { ctx.lex.store(t); }
          
          attrs.push(Self::read_attr_sub(ctx)?);

          let t = ctx.lex.get()?;
          if t.kind == WordKind::Comma { continue; }
          else if t.kind == WordKind::ParenEnd { break; }
          else {
            t.expect_equal_str2(",", ")")?;
          }
        }

        let this = Attr{vari: AttrVari::List(key, attrs)};

        Ok(ctx.cre.new_attr(this))
      }

      WordKind::Assign => { // A = $expr
        let ex = ExprParser::read_expr(ctx, 0)?;

        let this = Attr{vari: AttrVari::Set(key, ex)};

        Ok(ctx.cre.new_attr(this))
      }

      _ => {
        ctx.lex.store(_c);

        let this = Attr{vari: AttrVari::One(key)};

        return Ok(ctx.cre.new_attr(this));
      }
    
    }
  }


  pub fn attach_attr<'a, 'ctx, 'd, T>(ctx: &mut ParserContext<'a, 'ctx, 'd>, id: AstId<T>, attrs: Vec<AttrId>) {
    if !attrs.is_empty() {
      ctx.cre.map_attr.insert(AnyId::new_from(id), attrs);
    }
  }

}
