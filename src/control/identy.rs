use core::fmt;
use owo_colors::OwoColorize;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentyKind {
  Null = 0,

  Type = 1,
  Decl = 2,
  Expr = 3,
  Stmt = 4,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct IdentyId {
  module_and_kind: u32,
  m_index: u32,
}

impl IdentyId {

  pub fn new(kind: IdentyKind, module: u32, index: u32) -> Self {
    debug_assert!(module < (1 << 29), "Module ID 29 biti geçemez!");
    
    let packed = (module << 3) | (kind as u32);
    return Self {
      module_and_kind: packed,
      m_index: index,
    }
  }

  pub fn null() -> Self {
    Self{module_and_kind: 0, m_index:0}
  }

  pub fn kind(&self) -> IdentyKind {
    let kind_bits = (self.module_and_kind & 0b111) as u8;

    return match kind_bits {
      0 => IdentyKind::Null,
      1 => IdentyKind::Type,
      2 => IdentyKind::Decl,
      3 => IdentyKind::Expr,
      4 => IdentyKind::Stmt,
      _ => panic!(),
    };
  }

  pub fn module(&self) -> u32 {
    self.module_and_kind >> 3
  }

  pub fn index(&self) -> u32 {
    return self.m_index;
  }

}


impl fmt::Display for IdentyId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.kind() == IdentyKind::Null {
      write!(f, "({})", "!".bright_black())?;
      return Ok(());
    }
    
    let k = match self.kind() {
      IdentyKind::Type => "T",
      IdentyKind::Decl => "D",
      IdentyKind::Expr => "E",
      IdentyKind::Stmt => "S",
      _ => panic!(),
    };

    write!(f, "({}{}{:x}{}{:x})",
      k,
      ":".bright_black(),
      self.module(),
      ":".bright_black(),
      self.index()
    )?;
    Ok(())
  }
}
