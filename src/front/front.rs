use crate::{ast::{self, AccessKind, Crate, Module, Visibility}, diagnostic::{Message, Summary}, error, front::{attr_p::AttrParser, decl_p::DeclParser, item_p::ItemParser, meta_p::MetaParser}, lexer::{Lexer, WK, WordKind}, route::build::FileArena};

pub struct ParserContext<'a, 'ctx, 'd:'a> {
  pub lex: &'ctx mut Lexer<'a>,
  pub cre: &'ctx mut Crate<'a, 'd>,
  pub sum: &'ctx mut Summary<'a>,
  pub far: &'a FileArena,
  pub sides: Vec<ast::AnyId>,
}

impl<'a, 'ctx, 'd> ParserContext<'a, 'ctx, 'd> {
  
  pub fn new<'f>(front: &'ctx mut Front<'f, 'a, 'd>) -> ParserContext<'a, 'ctx, 'd> {
    ParserContext{
      lex: &mut front.lex,
      cre: &mut front.cre,
      sum: &mut front.sum,
      far: front.far,
      sides: vec![]
    }
  }

}



pub struct Front<'f, 'a, 'd:'a> {
  pub cre: &'f mut Crate<'a,'d>,
  pub far: &'a FileArena,
  pub lex: Lexer<'a>,
  pub sum: Summary<'a>,
}

impl<'f, 'a, 'd> Front<'f, 'a, 'd> {

  pub fn new(cre: &'f mut Crate<'a,'d>, mol: &'a Module, farena: &'a FileArena) -> error::Result<Front<'f, 'a, 'd>> {
    let lex = Lexer::new_module(mol);
    let sum = Summary::new();

    Ok(Front{cre, far: farena, lex, sum})
  }


  pub fn read<'ctx>(ctx: &mut ParserContext<'a, 'ctx, 'd>, defvis: &mut Visibility) -> Result<ast::AnyId, Message<'a>> { loop {
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
      },

      _ => return Err(Message::error(l, String::from("unknown keyword: `{}`"), vec![l.string()])),
    };

    AttrParser::attach_attr(ctx, id, attrs);
    return Ok(id)
  }}


  pub fn parse(&mut self, av: &mut Vec<ast::AnyId>) -> Summary<'a> {
    let mut pctx = ParserContext::new(self);
    let mut defvis = Visibility::Private;

    loop {
      let t = pctx.lex.lex();

      if t.kind == WordKind::EOF { break; } else { pctx.lex.store(t); }

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
