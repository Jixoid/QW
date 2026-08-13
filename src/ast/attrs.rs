use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast, lexer::Word};


#[derive(Clone)]
pub enum AttrVari<'a> {
  One(Word<'a>),
  Bin(Word<'a>, Word<'a>),
  Set(Word<'a>, ast::ExprId), 
  List(Word<'a>, Vec<ast::AttrId>),
}


#[derive(Clone)]
pub struct Attr<'a> {
  pub vari: AttrVari<'a>,
}


impl<'a> fmt::Display for Attr<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.vari {
      AttrVari::List(w, l) => {
        write!(f, "{}{}", w.str().yellow().bold(), "(".bright_black())?;
        
        for (i, x) in l.iter().enumerate() {
          write!(f, "{}", x)?;

          if i +1 < l.len() { write!(f, ", ")?; }
        }
        
        write!(f, "{}", ")".bright_black())?;
      }
    
      AttrVari::One(w) => {
        write!(f, "{}", w.str().yellow().bold())?;
      }

      AttrVari::Bin(k, v) => {
        write!(f, "{}{}{}", k.str().yellow().bold(), ":".bright_black(), v.str().yellow().bold())?;
      }

      AttrVari::Set(w, e) => {
        write!(f, "{} {} {}", w.str().yellow().bold(), "=".bright_black(), e)?;
      }

    };
    
    Ok(())
  }
}
