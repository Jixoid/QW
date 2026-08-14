use crate::hir::TypeId;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }


pub enum Type {
  Ptr{sub: TypeId, acc: AccessKind},
  Ref{sub: TypeId, acc: AccessKind},
}
