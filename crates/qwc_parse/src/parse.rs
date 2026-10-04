use qwc_arena::{File, Files};
use qwc_ast::{self as ast, ItemId};
use qwc_ast::{Krate, Visibility};
use qwc_diagnostic::{Span, Summary};
use qwc_lexer::Lexer;
use qwc_string_interner::StrInterner;

use crate::{ItemParser, MetaParser};


pub struct Ctx<'a,'d> {
  pub cre: &'a mut Krate,
  pub sin: &'a mut StrInterner,
  pub lex: &'a mut Lexer<'d>,
  pub sum: &'a mut Summary,
  pub far: &'a Files,
  pub side: &'a mut Vec<ItemId>,
}


#[macro_export]
macro_rules! ctx {
  ($ctx:expr => $cre:ident, $sin:ident, $far:ident, $lex:ident, $sum:ident, $side:ident) => {
    #[allow(unused_variables)]
    let Ctx{$cre, $sin, $far, $lex, $sum, $side} = $ctx;
  };
  
  ($cre:ident, $sin:ident, $far:ident, $lex:ident, $sum:ident, $side:ident) => {
    &mut Ctx{$cre, $sin, $far, $lex, $sum, $side}
  };
}


pub struct Parse;

impl Parse {

  pub fn parse(cre: &mut Krate, sin: &mut StrInterner, far: &Files, fi: &File) -> (Span, ast::ItemRng, Summary) {
    let lex = &mut Lexer::new(fi);
    let mut sum = Summary::new();
    
    let start = match lex.peek_safe() {
      Some(v) => v,
      None => return (lex.pos_extend_file(), ast::Rng::empty(), sum),
    };
    
    let rng = {
      let mut av = vec![];

      loop {
        if lex.peek_safe().is_none() { break }
        
        let mut side = vec![];

        // Any
        match ItemParser::read_item(&mut Ctx{cre, sin, far, lex, sum: &mut sum, side: &mut side}, &mut Visibility::Inherited) {
          Ok(aid) => {
            av.push(aid);
            av.extend(side);
          }
          
          Err(e) => {
            sum.add(e);
            if let Err(e) = MetaParser::pmr_global(&mut Ctx{cre, sin, far, lex, sum: &mut sum, side: &mut vec![]}) { sum.add(e) }
          }
        }
      }

      cre.extra(&av)
    };
    
    (lex.pos_extend(start), rng, sum)
  }

}
