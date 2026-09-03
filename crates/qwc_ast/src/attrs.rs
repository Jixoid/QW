use thin_vec::ThinVec;

use crate::{ExprId, Ident};


#[derive(Clone)]
pub enum Attribute {
  One(Ident),
  Bin(Ident, Ident),
  Set(Ident, ExprId), 
  List(Ident, ThinVec<Attribute>),
}
