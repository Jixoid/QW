#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecType;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecGlob;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecFunc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecBlok;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecInst;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct SpecValu;

pub type TypeId = MirId<SpecType>;
pub type GlobId = MirId<SpecGlob>;
pub type FuncId = MirId<SpecFunc>;
pub type BlokId = MirId<SpecBlok>;
pub type InstId = MirId<SpecInst>;
pub type ValuId = MirId<SpecValu>;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirKind {
  Type = 1,
  Glob = 2,
  Func = 3,
  Blok = 4,
  Inst = 5,
  Valu = 6,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct MirId<T> {
  pub kind: MirKind,
  pub krate: u16,
  pub index: u32,

  spec: Option<T>,
}


impl<T> MirId<T> {

  pub fn new(kind: MirKind, krate: u16, index: u32) -> Self {
    Self{ kind, krate, index, spec: None }
  }

}
