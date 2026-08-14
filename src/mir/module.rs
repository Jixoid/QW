use super::{identy::{HirId, HirKind}, types::HirType, global::HirGlobalVar, func::HirFunc, block::HirBlock, instr::HirInstr};

#[derive(Debug)]
pub struct HirModule {
  pub name: String,
  
  pub list_type: Vec<HirType>,
  pub list_global: Vec<HirGlobalVar>,
  pub list_func: Vec<HirFunc>,
  pub list_block: Vec<HirBlock>,
  pub list_instr: Vec<HirInstr>,
}

impl HirModule {
  pub fn new(name: String) -> Self {
    Self {
      name,
      list_type: Vec::new(),
      list_global: Vec::new(),
      list_func: Vec::new(),
      list_block: Vec::new(),
      list_instr: Vec::new(),
    }
  }

  pub fn get_type(&self, id: HirId) -> &HirType {
    debug_assert!(id.kind() == HirKind::Type);
    &self.list_type[id.index() as usize]
  }

  pub fn get_global(&self, id: HirId) -> &HirGlobalVar {
    debug_assert!(id.kind() == HirKind::Global);
    &self.list_global[id.index() as usize]
  }

  pub fn get_func(&self, id: HirId) -> &HirFunc {
    debug_assert!(id.kind() == HirKind::Func);
    &self.list_func[id.index() as usize]
  }
  
  pub fn get_func_mut(&mut self, id: HirId) -> &mut HirFunc {
    debug_assert!(id.kind() == HirKind::Func);
    &mut self.list_func[id.index() as usize]
  }

  pub fn get_block(&self, id: HirId) -> &HirBlock {
    debug_assert!(id.kind() == HirKind::Block);
    &self.list_block[id.index() as usize]
  }
  
  pub fn get_block_mut(&mut self, id: HirId) -> &mut HirBlock {
    debug_assert!(id.kind() == HirKind::Block);
    &mut self.list_block[id.index() as usize]
  }

  pub fn get_instr(&self, id: HirId) -> &HirInstr {
    debug_assert!(id.kind() == HirKind::Instr);
    &self.list_instr[id.index() as usize]
  }

  pub fn new_type(&mut self, v: HirType) -> HirId {
    self.list_type.push(v);
    HirId::new(HirKind::Type, 0, self.list_type.len() as u32 - 1)
  }

  pub fn new_global(&mut self, v: HirGlobalVar) -> HirId {
    self.list_global.push(v);
    HirId::new(HirKind::Global, 0, self.list_global.len() as u32 - 1)
  }

  pub fn new_func(&mut self, v: HirFunc) -> HirId {
    self.list_func.push(v);
    HirId::new(HirKind::Func, 0, self.list_func.len() as u32 - 1)
  }

  pub fn new_block(&mut self, v: HirBlock) -> HirId {
    self.list_block.push(v);
    HirId::new(HirKind::Block, 0, self.list_block.len() as u32 - 1)
  }

  pub fn new_instr(&mut self, v: HirInstr) -> HirId {
    self.list_instr.push(v);
    HirId::new(HirKind::Instr, 0, self.list_instr.len() as u32 - 1)
  }
}

use core::fmt;
use owo_colors::OwoColorize;

impl fmt::Display for HirModule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  {}{} {}b", "types".purple().bold(), ":".bright_black(), std::mem::size_of::<HirType>() * self.list_type.len())?;
    for (i, x) in self.list_type.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.vari)?;
    }
    writeln!(f)?;

    writeln!(f, "  {}{} {}b", "globals".purple().bold(), ":".bright_black(), std::mem::size_of::<HirGlobalVar>() * self.list_global.len())?;
    for (i, x) in self.list_global.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;

    writeln!(f, "  {}{} {}b", "funcs".purple().bold(), ":".bright_black(), std::mem::size_of::<HirFunc>() * self.list_func.len())?;
    for (i, x) in self.list_func.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;

    writeln!(f, "  {}{} {}b", "blocks".purple().bold(), ":".bright_black(), std::mem::size_of::<HirBlock>() * self.list_block.len())?;
    for (i, x) in self.list_block.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;

    writeln!(f, "  {}{} {}b", "instrs".purple().bold(), ":".bright_black(), std::mem::size_of::<HirInstr>() * self.list_instr.len())?;
    for (i, x) in self.list_instr.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;

    Ok(())
  }
}
