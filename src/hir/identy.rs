#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecType;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecDecl;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecExpr;

pub type TypeId = HirId<SpecType>;
pub type DeclId = HirId<SpecDecl>;
pub type ExprId = HirId<SpecExpr>;



#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum HirKind {
  Type = 1,
  Decl = 2,
  Expr = 3,
}


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct HirId<T> {
  pub kind: HirKind,
  pub krate: u16,
  pub index: u32,

  spec: Option<T>,
}

impl<T> HirId<T> {

  pub fn new(kind: HirKind, krate: u16, index: u32) -> Self {
    Self{ kind, krate, index, spec: None }
  }

}
