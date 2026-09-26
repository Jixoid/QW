use crate::{ExprId, ExprRng, Ident, ThingRng, TypeId, TypeRng, Visibility};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Thing {
  Name(Ident),
  List(ThingRng),

  Wildcard, Crate, Super,
  
  NamedExpr(Ident, ExprId),
  NamedType(Ident, TypeId),
  
  TypeVis(TypeId, Visibility),
  
  NamedTypeVis(Ident, Visibility, TypeId),

  NamedTypeList(Ident, TypeRng),
  NamedExprList(Ident, ExprRng),

  MatchArm(ExprId /* pat */, ExprId /* body */),
}
