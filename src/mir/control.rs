use crate::mir::{Blok, BlokId, Func, FuncId, Glob, GlobId, Inst, InstId, MirId, MirKind, Type, TypeId};


pub struct Crate {
  pub list_type: Vec<Type>,
  pub list_glob: Vec<Glob>,
  pub list_func: Vec<Func>,
  pub list_blok: Vec<Blok>,
  pub list_inst: Vec<Inst>,
}

impl Crate {

  pub fn new() -> Self {
    Self {
      list_type: vec![],
      list_glob: vec![],
      list_func: vec![],
      list_blok: vec![],
      list_inst: vec![],
    }
  }


  pub fn get_type(&self, id: TypeId) -> &Type {
    debug_assert!(id.kind == MirKind::Type);
    &self.list_type[id.index as usize]
  }

  pub fn get_glob(&self, id: GlobId) -> &Glob {
    debug_assert!(id.kind == MirKind::Glob);
    &self.list_glob[id.index as usize]
  }

  pub fn get_func(&self, id: FuncId) -> &Func {
    debug_assert!(id.kind == MirKind::Func);
    &self.list_func[id.index as usize]
  }
  
  pub fn get_blok(&self, id: BlokId) -> &Blok {
    debug_assert!(id.kind == MirKind::Blok);
    &self.list_blok[id.index as usize]
  }
  
  pub fn get_inst(&self, id: InstId) -> &Inst {
    debug_assert!(id.kind == MirKind::Inst);
    &self.list_inst[id.index as usize]
  }


  pub fn new_type(&mut self, v: Type) -> TypeId {
    self.list_type.push(v);
    MirId::new(MirKind::Type, 0, self.list_type.len() as u32 - 1)
  }

  pub fn new_glob(&mut self, v: Glob) -> GlobId {
    self.list_glob.push(v);
    MirId::new(MirKind::Glob, 0, self.list_glob.len() as u32 - 1)
  }

  pub fn new_func(&mut self, v: Func) -> FuncId {
    self.list_func.push(v);
    MirId::new(MirKind::Func, 0, self.list_func.len() as u32 - 1)
  }

  pub fn new_blok(&mut self, v: Blok) -> BlokId {
    self.list_blok.push(v);
    MirId::new(MirKind::Blok, 0, self.list_blok.len() as u32 - 1)
  }

  pub fn new_inst(&mut self, v: Inst) -> InstId {
    self.list_inst.push(v);
    MirId::new(MirKind::Inst, 0, self.list_inst.len() as u32 - 1)
  }


  pub fn storage_size(&self) -> usize {
    size_of::<Type>()*self.list_type.len()
    +
    size_of::<Glob>()*self.list_glob.len()
    +
    size_of::<Func>()*self.list_func.len()
    +
    size_of::<Blok>()*self.list_blok.len()
    +
    size_of::<Inst>()*self.list_inst.len()
  }

}
