use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HirKind {
  Null = 0,
  Type = 1,
  Global = 2,
  Func = 3,
  Block = 4,
  Instr = 5,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct HirId {
  module_and_kind: u32,
  m_index: u32,
}

impl HirId {

  pub fn new(kind: HirKind, module: u32, index: u32) -> Self {
    debug_assert!(module < (1 << 29), "Module ID cannot exceed 29 bits!");
    
    let packed = (module << 3) | (kind as u32);
    Self {
      module_and_kind: packed,
      m_index: index,
    }
  }

  pub fn null() -> Self {
    Self { module_and_kind: 0, m_index: 0 }
  }

  pub fn kind(&self) -> HirKind {
    let kind_bits = (self.module_and_kind & 0b111) as u8;
    match kind_bits {
      0 => HirKind::Null,
      1 => HirKind::Type,
      2 => HirKind::Global,
      3 => HirKind::Func,
      4 => HirKind::Block,
      5 => HirKind::Instr,
      _ => unreachable!(),
    }
  }

  pub fn module(&self) -> u32 {
    self.module_and_kind >> 3
  }

  pub fn index(&self) -> u32 {
    self.m_index
  }

}

impl fmt::Display for HirId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.kind() == HirKind::Null {
      write!(f, "(!)")?;
      return Ok(());
    }
    
    let k = match self.kind() {
      HirKind::Type   => "HT",
      HirKind::Global => "HG",
      HirKind::Func   => "HF",
      HirKind::Block  => "HB",
      HirKind::Instr  => "HI",
      _ => unreachable!(),
    };

    write!(f, "({}:{:x}:{:x})", k, self.module(), self.index())?;
    Ok(())
  }
}

