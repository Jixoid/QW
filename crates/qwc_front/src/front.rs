use qwc_arena::{File, Files};
use qwc_ast::{self as ast, StrInterner};
use qwc_ast::{Krate, Visibility};
use qwc_diagnostic::{Span, Summary};
use qwc_lexer::Lexer;

use crate::{ItemParser, MetaParser};


pub struct Ctx<'a,'d> {
  pub cre: &'a mut Krate,
  pub sin: &'a mut StrInterner,
  pub lex: &'a mut Lexer<'d>,
  pub sum: &'a mut Summary,
  pub far: &'a Files,
}


#[macro_export]
macro_rules! ctx {
  ($ctx:expr => $cre:ident, $sin:ident, $far:ident, $lex:ident, $sum:ident) => {
    #[allow(unused_variables)]
    let Ctx{$cre, $sin, $far, $lex, $sum} = $ctx;
  };
  
  ($cre:ident, $sin:ident, $far:ident, $lex:ident, $sum:ident) => {
    &mut Ctx{$cre, $sin, $far, $lex, $sum}
  };
}


pub struct Front;

impl Front {

  pub fn parse(cre: &mut Krate, sin: &mut StrInterner, far: &Files, fi: &File) -> (Option<(Span, ast::Rng)>, Summary) {
    let lex = &mut Lexer::new(fi);
    let mut sum = Summary::new();
    
    let start = match lex.peek() {
      Ok(v) => v,
      Err(e) => { sum.add(e); return (None, sum) },
    };
    
    let rng = {
      let mut av = vec![];

      loop {
        if lex.peek_safe().is_none() { break }
        
        // Any
        match ItemParser::read_item(&mut Ctx{cre, sin, far, lex, sum: &mut sum}, &mut Visibility::Inherited) {
          Ok(aid) => av.push(aid),
          
          Err(e) => {
            sum.add(e);
            if let Err(e) = MetaParser::pmr_global(&mut Ctx{cre, sin, far, lex, sum: &mut sum}) { sum.add(e) }
          }
        }
      }

      cre.extra(&av)
    };
    
    (Some((lex.pos_extend(start), rng)), sum)
  }

}
