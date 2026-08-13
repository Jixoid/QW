use core::fmt;

use crate::{ast, lexer::Word};


#[derive(Clone)]
pub enum DirceVari<'a> {
  One(Word<'a>),
  Use(Word<'a>),
  Feat(Word<'a>),
  Set{key: Word<'a>, val: ast::ExprId},
  Cfg(ast::ExprId),
}

#[derive(Clone)]
pub struct Dirc<'a> {
  pub vari: DirceVari<'a>,
}


impl<'a> fmt::Display for Dirc<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.vari {
      DirceVari::One(w) => write!(f, "{}", w.str())?,
      DirceVari::Use(w) => write!(f, "use {}", w.str())?,
      DirceVari::Feat(w) => write!(f, "feat {}", w.str())?,
      DirceVari::Set{key, val} => write!(f, "{} = {}", key.str(), val)?,
      DirceVari::Cfg(e) => write!(f, "cfg({})", e)?,
    }

    Ok(())
  }
}
