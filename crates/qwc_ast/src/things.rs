use crate::{ExprId, Ident, Rng, TypeId, Visibility};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Thing {
  Name(Ident),
  List(Rng),

  Wildcard, Crate, Super,
  
  NamedExpr(Ident, ExprId),
  NamedType(Ident, TypeId),
  
  TypeVis(TypeId, Visibility),
  
  NamedTypeVis(Ident, Visibility, TypeId),

  NamedTypeList(Ident, Rng /* TypeId */),
  NamedExprList(Ident, Rng /* ExprId */),

  MatchArm(ExprId /* pat */, ExprId /* body */),
}
