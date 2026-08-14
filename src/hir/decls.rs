use crate::hir::{AccessKind, ExprId, TypeId};


pub enum Decl {
  Var  {kind: TypeId, acck: AccessKind, init: Option<ExprId>},
  Fun  {kind: TypeId, blok: ExprId},
  Using{kind: TypeId},
}
