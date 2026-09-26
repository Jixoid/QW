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


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Value {
  SSA(SSA),
  Const(Const),
  GlobalRef(SymbId),
  StackRef(u32),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Const {
  // ZST
  Unit,
  
  // Primitive
  Bool(bool),
  Int(i32),
}


#[derive(Debug, Copy, Clone)]
pub enum Expr {
  Binary(Value, Value),

  Store{target: Value, kind: TypeId, value: Value},
  Load{target: Value, kind: TypeId},
}

impl Expr {
  pub fn have_result(&self) -> bool {
    match self {
      Expr::Binary(..) => true,
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
