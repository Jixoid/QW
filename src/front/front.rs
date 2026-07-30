use crate::{ast::Visibility, control::{Module, ModuleFile, module}, diagnostic::Summary, front::{decl_p::DeclParser, meta_p::MetaParser}, lexer::{Lexer, WordKind}, route::build::ModInjection};

pub struct ParserContext<'a, 'ctx, 'd:'a> {
  pub lex: &'ctx mut Lexer<'a>,
  pub mol: &'ctx mut Module<'a, 'd>,
  pub sum: &'ctx mut Summary<'a>,
  pub mi: &'ctx ModInjection<'d>,
}

impl<'a, 'ctx, 'd> ParserContext<'a, 'ctx, 'd> {
  
  pub fn new<'f>(front: &'ctx mut Front<'f, 'a, 'd>, mi: &'ctx ModInjection<'d>) -> ParserContext<'a, 'ctx, 'd> {
    ParserContext{
      lex: &mut front.lex,
      mol: &mut front.mol,
      sum: &mut front.sum,
      mi,
    }
  }

}



pub struct Front<'f, 'a, 'd:'a> {
  pub mol: &'f mut Module<'a,'d>,
  pub lex: Lexer<'a>,
  pub sum: Summary<'a>,
}

impl<'f, 'a, 'd> Front<'f, 'a, 'd> {

  pub fn new(mol: &'f mut Module<'a,'d>, mfd: &'a ModuleFile) -> module::Result<Front<'f, 'a, 'd>> {
    let lex = Lexer::new_module(mfd);
    let sum = Summary::new();

    Ok(Front{mol, lex, sum})
  }

  pub fn parse(&mut self, mi: &'d ModInjection<'d>) -> Summary<'a> {
    let mut pctx = ParserContext::new(self, mi);
    let mut defvis = Visibility::Private;

    loop {
      let t = pctx.lex.lex();

      if t.kind == WordKind::EOF { break; }
      else {
        pctx.lex.store(t);

        match DeclParser::read_decl(&mut pctx, &mut defvis) {
          Ok(Some(r)) => { pctx.mol.add_to_module(r); },
          Ok(None) => {},
          Err(e) => {
            pctx.sum.add(e);

            match MetaParser::pmr_global(&mut pctx) {
              Ok(()) => (),
              Err(e) => pctx.sum.add(e),
            };
          }
        };
      }
    }

    return self.sum.clone();
  }

}

