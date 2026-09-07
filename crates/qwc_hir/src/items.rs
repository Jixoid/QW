use qwc_string_interner::Sid;

use crate::{ExprId, Rng, TypeId};


#[derive(Debug, Copy, Clone)]
pub enum Item {
  RootNS{rng: Rng /* ItemId */},
  NameSpace{rng: Rng /* ItemId */, name: Sid},
  GenericNS{rng: Rng /* ItemId */},

  Variable{kind: TypeId, expr: ExprId, name: Sid, ism: bool},
  Function{kind: TypeId, expr: ExprId, name: Sid},
}
