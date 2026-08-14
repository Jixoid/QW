use crate::{
  ast::{self, AccessKind, Crate, Module, Visibility}, diagnostic::{Message, Summary}, error, lexer::{Lexer, WK},
  front::{attr_p::AttrParser, decl_p::DeclParser, item_p::ItemParser, meta_p::MetaParser}, route::build::FileArena
};

pub struct ParserContext<'a, 'ctx> {
  pub lex: &'ctx mut Lexer<'a>,
  pub cre: &'ctx mut Crate,
  pub sum: &'ctx mut Summary,
  pub far: &'a FileArena,
  pub sides: Vec<ast::AnyId>,
}

impl<'a,'ctx> ParserContext<'a,'ctx> {
  
  pub fn new<'f>(front: &'ctx mut Front<'f,'a>) -> ParserContext<'a,'ctx> {
    ParserContext{
      lex: &mut front.lex,
      cre: &mut front.cre,
      sum: &mut front.sum,
      far: front.far,
      sides: vec![]
    }
  }

}



pub struct Front<'f, 'a> {
  pub cre: &'f mut Crate,
  pub far: &'a FileArena,
  pub lex: Lexer<'a>,
  pub sum: Summary,
}

impl<'f,'a,'d> Front<'f,'a> {

  pub fn new(cre: &'f mut Crate, mol: &'a Module, farena: &'a FileArena) -> error::Result<Front<'f,'a>> {
    let lex = Lexer::new(mol);
    let sum = Summary::new();

    Ok(Front{cre, far: farena, lex, sum})
  }


  pub fn read<'ctx>(ctx: &mut ParserContext<'a, 'ctx>, defvis: &mut Visibility) -> Result<ast::AnyId, Message> { loop {
    let attrs = AttrParser::read_attr(ctx)?;
    let v = MetaParser::read_visibility(ctx, defvis)?;

    let l = ctx.lex.get()?;
    let id = match l.kind {
      WK::Fun    => DeclParser::read_fun(ctx, v, false)?.to_any(),
      WK::Using  => DeclParser::read_using(ctx, v)?.to_any(),
      WK::Struct => DeclParser::read_struct(ctx, v)?.to_any(),
      WK::Iface  => DeclParser::read_iface(ctx, v)?.to_any(),
      WK::Trait  => DeclParser::read_trait(ctx, v)?.to_any(),
      WK::Enum   => DeclParser::read_enum(ctx, v)?.to_any(),
      WK::Flags  => DeclParser::read_flags(ctx, v)?.to_any(),
      WK::Let    => DeclParser::read_var(ctx, v, AccessKind::IMM)?.to_any(),
      WK::Var    => DeclParser::read_var(ctx, v, AccessKind::MUT)?.to_any(),
      
      WK::Impl    => ItemParser::read_impl(ctx, v)?.to_any(),
      WK::Generic => ItemParser::read_generic(ctx, v)?.to_any(),
      
      //"use"    => return Self::read_use(ctx, v),
      WK::Mod     => match ItemParser::read_mod(ctx, v)? {
        Some(r) => r.to_any(),
        None => continue,
      }

      _ => return Err(Message::error(l, String::from("unknown keyword: `{}`"), vec![l.string(ctx.far)])),
    };

    AttrParser::attach_attr(ctx, id, attrs);
    return Ok(id)
  }}


  pub fn parse(&mut self, av: &mut Vec<ast::AnyId>) -> Summary {
    let mut pctx = ParserContext::new(self);
    let mut defvis = Visibility::Private;

    loop {
      match pctx.lex.lex() {
        None => break,
        Some(r) => pctx.lex.store(r),
      }

      // Any
      match Self::read(&mut pctx, &mut defvis) {
        Ok(aid) => {
          av.push(aid);
          av.extend(pctx.sides.drain(..));
        }
        
        Err(e) => {
          pctx.sum.add(e);
          if let Err(e) = MetaParser::pmr_global(&mut pctx) { pctx.sum.add(e) }
        }
      }
    }

    return self.sum.clone();
  }

}
