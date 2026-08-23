#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecAny;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecType;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecExpr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecItem;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecThing;

pub type AnyId  = HirId<SpecAny>;
pub type TypeId = HirId<SpecType>;
pub type ExprId = HirId<SpecExpr>;
pub type ItemId = HirId<SpecItem>;
pub type ThingId = HirId<SpecThing>;



#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum HirKind {
  Type = 1,
  Expr = 2,
  Item = 3,
  Thing = 4,
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

  pub fn to_any(&self) -> AnyId {
    AnyId{ kind: self.kind, krate: self.krate, index: self.index, spec: None }
  }

}

impl<T> HirId<T> {
  
  pub fn new_from<J>(id: HirId<J>) -> Self {
    Self{ kind: id.kind, krate: id.krate, index: id.index, spec: None }
  }

}
