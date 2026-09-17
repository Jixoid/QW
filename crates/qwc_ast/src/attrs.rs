use thin_vec::ThinVec;

use crate::{ExprId, Ident};


#[derive(Clone)]
pub enum AttrKind {
  One(),
  Bin(Ident),
  Set(ExprId), 
  List(ThinVec<Attribute>),
}

#[derive(Clone)]
pub struct Attribute {
  pub ident: Ident,
  pub kind: AttrKind,
}
