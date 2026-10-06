use qwc_string_interner::Sid;

use crate::{Const, ExprId, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Thing {
  NamedType(Sid, TypeId),
  NamedExpr(Sid, ExprId),
  NamedConst(Sid, Const),
}
