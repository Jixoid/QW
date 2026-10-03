use qwc_string_interner::Sid;

use crate::{ExprId, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Thing {
  NamedExpr(Sid, ExprId),
  NamedType(Sid, TypeId),
}
