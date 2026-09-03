use crate::{ast, hir::{ExprId, Rng, TypeId}};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }

impl From<ast::AccessKind> for AccessKind {
  fn from(value: ast::AccessKind) -> Self {
    match value {
      ast::AccessKind::IMM => AccessKind::IMM,
      ast::AccessKind::MUT => AccessKind::MUT,
    }
  }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
  Ptr(TypeId, AccessKind),
  Ref(TypeId, AccessKind),
  DynArr(TypeId),
  StaArr(TypeId, u64),

  Range(TypeId),
  Option(TypeId),
  Result(TypeId, TypeId),
  Vector(TypeId, ExprId),

  Unit, Bool, Char, Str,

  MetaType,
  
  Int(u16, bool), ArchInt(bool),
  Float(u16), ArchFloat,

  Struct(Rng /* NamedTypeVis */),
  Tuple (Rng /* TypeId */),

  Iface (Rng /* NamedTypeVis */),
  Trait (Rng /* NamedTypeVis */),

  Enum (Rng /* NamedExpr */),
  Flags(Rng /* NamedExpr */),

  Fun{args: Rng /* NamedType */, ret: Option<TypeId>},
}
