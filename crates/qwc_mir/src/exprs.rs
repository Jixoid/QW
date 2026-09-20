use std::fmt;

use owo_colors::OwoColorize;

use crate::{SymbId, TypeId};



#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SSA ( pub(crate) u32 );

impl SSA {
  pub fn new(val: u32) -> Self { Self(val) }
}

impl fmt::Display for SSA {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", format!("%{}", self.0).purple().bold()) }
}


#[derive(Debug, Copy, Clone)]
pub enum Const {
  // ZST
  Unit,
  
  // Primitive
  Bool(bool),
  Int(i32),
}


#[derive(Debug, Copy, Clone)]
pub enum Expr {
  Const(Const),
  
  Use(SSA),
  
  Binary(SSA, SSA),

  GlobalRef(SymbId),

  Store{target: SSA, kind: TypeId, value: SSA},
  Load{target: SSA, kind: TypeId},
}

impl Expr {
  pub fn have_result(&self) -> bool {
    match self {
      Expr::Const(..) => true,
      Expr::Use(..) => true,
      Expr::Binary(..) => true,
      Expr::GlobalRef(..) => true,
      Expr::Load{..} => true,

      Expr::Store{..} => false,
    }
  }
}


#[derive(Debug, Copy, Clone)]
pub struct Inst {
  pub kind: Expr,
  pub dest: Option<SSA>,
}
