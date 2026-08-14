#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecAny;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecType;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecDecl;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecExpr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecAttr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecDirc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecItem;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecPatt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecThing;

pub type AnyId  = AstId<SpecAny>;
pub type TypeId = AstId<SpecType>;
pub type DeclId = AstId<SpecDecl>;
pub type ExprId = AstId<SpecExpr>;
pub type AttrId = AstId<SpecAttr>;
pub type DircId = AstId<SpecDirc>;
pub type ItemId = AstId<SpecItem>;
pub type PattId = AstId<SpecPatt>;
pub type ThingId = AstId<SpecThing>;



#[repr(u8)]
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum AstKind {
  Null  = 0,
  Type  = 1,
  Decl  = 2,
  Expr  = 3,
  Attr  = 4,
  Dirc  = 5,
  Item  = 6,
  Patt  = 7,
  Thing = 8,
}


#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct AstId<T> {
  pub kind: AstKind,
  pub krate: u16,
  pub index: u32,

  spec: Option<T>,
}

impl<T> AstId<T> {

  pub fn new(kind: AstKind, krate: u16, index: u32) -> Self {
    Self{ kind, krate, index, spec: None }
  }

  pub fn null() -> Self {
    Self{ kind: AstKind::Null, krate: 0, index: 0, spec: None }
  }

  pub fn is_null(&self) -> bool {
    let res = self.kind == AstKind::Null;
    
    if res {
      debug_assert_eq!(self.krate, 0);
      debug_assert_eq!(self.index, 0);
    }

    res
  }

  pub fn to_any(&self) -> AnyId {
    AnyId{ kind: self.kind, krate: self.krate, index: self.index, spec: None }
  }

}

impl<T> AstId<T> {
  
  pub fn new_from<J>(id: AstId<J>) -> Self {
    Self{ kind: id.kind, krate: id.krate, index: id.index, spec: None }
  }

}
