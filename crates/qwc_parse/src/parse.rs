/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_arena::{File, Files};
use qwc_ast::{self as ast, ItemId};
use qwc_ast::{Krate, Visibility};
use qwc_diagnostic::{Span, Status::{Fail, Success}, Summary};
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
  
  ($cre:expr, $sin:expr, $far:expr, $lex:expr, $sum:expr, $side:expr) => {
    &mut Ctx{cre: $cre, sin: $sin, far: $far, lex: $lex, sum: $sum, side: $side}
  };
}


pub struct Parse;

impl Parse {

  pub fn parse(cre: &mut Krate, base_path: ast::DefPathId, sin: &mut StrInterner, far: &Files, fi: &File) -> (Span, ast::ItemRng, Summary) {
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
        match ItemParser::read_item(&mut Ctx{cre, sin, far, lex, sum: &mut sum, side: &mut side}, base_path, &mut Visibility::Inherited) {
          Success(aid) => {
            av.push(aid);
            av.extend(side);
          }
          
          Fail => {
            let _ = MetaParser::pmr_global(&mut Ctx{cre, sin, far, lex, sum: &mut sum, side: &mut vec![]});
          }
        }
      }

      cre.extra(&av)
    };
    
    (lex.pos_extend(start), rng, sum)
  }

}
