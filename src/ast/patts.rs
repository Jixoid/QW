use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast::{self, Rng}, lexer::Word};


pub enum Patt<'a> {
  One(Word<'a>),
  
  Wildcard,
  Rest,
  
  Tuple(Rng),
  Array(Rng),
}



impl<'a> Patt<'a> {
  pub fn display<'m>(&'a self, module: &'m ast::Crate) -> PattDisplay<'a, 'm> {
    PattDisplay(self, module)
  }
}

pub struct PattDisplay<'a, 'm>(pub &'a Patt<'a>, pub &'m ast::Crate<'m,'m>);

impl<'a,'m> fmt::Display for PattDisplay<'a,'m> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let pt = self.0;
    let mol = self.1;

    match &pt {
      Patt::One(w) => write!(f, "{}", w.str().yellow().bold())?,
      
      Patt::Wildcard => write!(f, "{}", "_".bright_black())?,
      Patt::Rest => write!(f, "{}", "..".bright_black())?,

      Patt::Tuple(rng) => {
        let v = mol.get_extra(rng.clone());
        
        write!(f, "{}", "(".bright_black())?;

        for (i, x) in v.iter().enumerate() {
          write!(f, "{}", x)?;

          if i+1 < v.len() { write!(f, "{}", ",".bright_black())?; }
        }

        write!(f, "{}", ")".bright_black())?;
      }

      Patt::Array(rng) => {
        let v = mol.get_extra(rng.clone());

        write!(f, "{}", "[".bright_black())?;

        for (i, x) in v.iter().enumerate() {
          write!(f, "{}", x)?;

          if i+1 < v.len() { write!(f, "{}", ",".bright_black())?; }
        }

        write!(f, "{}", "]".bright_black())?;
      }
    }

    Ok(())
  }
}
