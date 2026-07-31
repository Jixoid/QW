use core::fmt;
use owo_colors::OwoColorize;
use std::{collections::HashMap, fs, path::Path, ptr};

use crate::{ast::*, control::{IdentyId, IdentyKind}};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind { Regular, RTL }


#[derive(Debug)]
pub enum CompilerError {
  Io(std::io::Error),
  Ds(String),
  Str(String),
}

impl From<std::io::Error> for CompilerError {
  fn from(err: std::io::Error) -> Self {
    CompilerError::Io(err)
  }
}

impl From<String> for CompilerError {
  fn from(err: String) -> Self {
    CompilerError::Str(err)
  }
}

impl fmt::Display for CompilerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CompilerError::Io(e)  => write!(f, "io: {}", e),
      CompilerError::Ds(e) => write!(f, "ds: {}", e),
      _ => write!(f, "unkown"),
    }
  }
}


pub type Result<T> = std::result::Result<T, CompilerError>;


#[derive(Debug)]
pub struct Module<'a, 'd:'a> {
  pub name: String,
  pub kind: ModuleKind,
  pub imod: Vec<&'d Module<'d,'d>>,

  pub nick_map: Vec<String>,

  pub list_type: Vec<Type<'a>>,
  pub list_decl: Vec<Decl<'a>>,
  pub list_expr: Vec<Expr<'a>>,
  pub list_stmt: Vec<Stmt<'a>>,

  pub map_attr: HashMap<IdentyId, Vec<Attribute<'a>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleFile {
  pub fpath: String,
  pub mmap: String,
  pub kind: ModuleKind,
}


impl<'a,'d> Module<'a,'d> {

  pub fn new(fpath: String, kind: ModuleKind) -> Result<(Module<'a,'d>, ModuleFile)> {
    let mmap = fs::read_to_string(&fpath)?;
    let name = String::from(Path::new(&fpath).file_stem().and_then(|s| s.to_str()).unwrap_or(""));
    
    let mut mol = Module{
      name: name.clone(), kind,
      imod: Vec::new(),
      nick_map: Vec::new(),
      list_type: vec![],
      list_decl: vec![],
      list_expr: vec![],
      list_stmt: vec![],
      map_attr: std::collections::HashMap::new(),
    };

    let mfd = ModuleFile{
      fpath,
      mmap,
      kind,
    };


    let molv = DeclVari::Module(ModuleDecl{decls: Vec::new() });
    let mold = Decl::new_str(name, molv,  Visibility::Public);

    mol.new_decl(mold);

    Ok((mol, mfd))
  }

  pub fn new_rtl(name: String) -> Result<Module<'a,'d>> {
    let mut mol = Module{
      name: name.clone(),
      kind: ModuleKind::RTL,
      imod: Vec::new(),
      nick_map: Vec::new(),
      list_type: vec![],
      list_decl: vec![],
      list_expr: vec![],
      list_stmt: vec![],
      map_attr: std::collections::HashMap::new(),
    };
    
    let molv = DeclVari::Module(ModuleDecl{decls: Vec::new() });
    let mold = Decl::new_str(name.clone(), molv, Visibility::Public);

    mol.new_decl(mold);

    Ok(mol)
  }


  pub fn add_to_module(&mut self, id: IdentyId) {
    if let DeclVari::Module(ref mut m) = self.list_decl[0].vari {
      m.decls.push(id);
    }
  }


  pub fn add_dep(&mut self, mol: &'d Module<'d,'d>) -> u32 {
    match self.imod.iter().position(|&x| ptr::eq(x, mol)) {
      Some(r) => (r as u32) +1,
      None => {
        self.imod.push(mol);
        self.imod.len() as u32
      }
    }
  }

  pub fn get_dep(&self, mod_id: u32) -> &'d Module<'d,'d> {
    self.imod[(mod_id -1) as usize] 
  }


  pub fn localize(&mut self, mol: &'d Module<'d,'d>, eid: IdentyId) -> IdentyId {
    if eid.module() == 1 { return eid; }

    let omol: &Module<'d,'d> = if eid.module() == 0 {
      mol
    } else {
      mol.get_dep(eid.module())
    };

    let lmid = self.add_dep(omol);

    IdentyId::new(eid.kind(), lmid, eid.index())
  }

  
  pub fn get_mod(&self) -> IdentyId {
    IdentyId::new(IdentyKind::Decl, 0,0)
  }


  pub fn get_type(&self, id: IdentyId) -> &Type<'a> {
    debug_assert!(id.kind() == IdentyKind::Type);
    
    let modl: &Module<'a,'a> = if id.module() == 0 { self } else { &self.imod[(id.module()-1) as usize] };

    &modl.list_type[id.index() as usize]
  }

  pub fn get_decl(&self, id: IdentyId) -> &Decl<'a> {
    debug_assert!(id.kind() == IdentyKind::Decl);
    
    let modl: &Module<'a,'a> = if id.module() == 0 { self } else { &self.imod[(id.module()-1) as usize] };

    &modl.list_decl[id.index() as usize]
  }

  pub fn get_expr(&self, id: IdentyId) -> &Expr<'a> {
    debug_assert!(id.kind() == IdentyKind::Expr);
    
    let modl: &Module<'a,'a> = if id.module() == 0 { self } else { &self.imod[(id.module()-1) as usize] };

    &modl.list_expr[id.index() as usize]
  }

  pub fn get_stmt(&self, id: IdentyId) -> &Stmt<'a> {
    debug_assert!(id.kind() == IdentyKind::Stmt);
    
    let modl: &Module<'a,'a> = if id.module() == 0 { self } else { &self.imod[(id.module()-1) as usize] };

    &modl.list_stmt[id.index() as usize]
  }


  pub fn get_mut_type(&mut self, id: IdentyId) -> &mut Type<'a> {
    assert_eq!(id.module(), 0, "Cannot mutate types from another module");
    &mut self.list_type[id.index() as usize]
  }

  pub fn get_mut_decl(&mut self, id: IdentyId) -> &mut Decl<'a> {
    debug_assert!(id.kind() == IdentyKind::Decl);
    assert_eq!(id.module(), 0, "Cannot mutate decl from another module");
    &mut self.list_decl[id.index() as usize]
  }

  pub fn get_mut_expr(&mut self, id: IdentyId) -> &mut Expr<'a> {
    debug_assert!(id.kind() == IdentyKind::Expr);
    debug_assert!(id.module() == 0, "Cannot get mut expr from other modules");
    &mut self.list_expr[id.index() as usize]
  }

  pub fn get_mut_stmt(&mut self, id: IdentyId) -> &mut Stmt<'a> {
    debug_assert!(id.kind() == IdentyKind::Stmt);
    debug_assert!(id.module() == 0, "Cannot mutate stmt from another module");
    &mut self.list_stmt[id.index() as usize]
  }


  pub fn new_type(&mut self, v: Type<'a>) -> IdentyId {
    if let Some(idx) = self.list_type.iter().position(|t| t.vari == v.vari) {
      return IdentyId::new(IdentyKind::Type, 0, idx as u32);
    }
    
    self.list_type.push(v);

    let idx = self.list_type.len() as u32 - 1;

    return IdentyId::new(IdentyKind::Type, 0, idx);
  }

  pub fn new_decl(&mut self, v: Decl<'a>) -> IdentyId {
    self.list_decl.push(v);

    let idx = self.list_decl.len() as u32 - 1;

    return IdentyId::new(IdentyKind::Decl, 0, idx);
  }

  pub fn new_expr(&mut self, v: Expr<'a>) -> IdentyId {
    self.list_expr.push(v);

    let idx = self.list_expr.len() as u32 - 1;

    return IdentyId::new(IdentyKind::Expr, 0, idx);
  }

  pub fn new_stmt(&mut self, v: Stmt<'a>) -> IdentyId {
    self.list_stmt.push(v);

    let idx = self.list_stmt.len() as u32 - 1;

    return IdentyId::new(IdentyKind::Stmt, 0, idx);
  }

}


impl<'a,'d> fmt::Display for Module<'a,'d> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  {}{}", "inter-module".purple().bold(), ":".bright_black())?;
    for (i, x) in self.imod.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i+1, "]".bright_black(), x.name)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "str-pool".purple().bold(), ":".bright_black())?;
    for (i, x) in self.nick_map.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{} {}b", "types".purple().bold(), ":".bright_black(), size_of::<Type>()*self.list_type.len())?;
    for (i, x) in self.list_type.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.display(self))?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{} {}b", "decls".purple().bold(), ":".bright_black(), size_of::<Decl>()*self.list_decl.len())?;
    for (i, x) in self.list_decl.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{} {}b", "exprs".purple().bold(), ":".bright_black(), size_of::<Expr>()*self.list_expr.len())?;
    for (i, x) in self.list_expr.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.display(self))?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{} {}b", "stmts".purple().bold(), ":".bright_black(), size_of::<Stmt>()*self.list_stmt.len())?;
    for (i, x) in self.list_stmt.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.display(self))?;
    }
    writeln!(f)?;


    for x in &self.imod {
      writeln!(f, "{}{} {}", "ast-dump".purple().bold(), ":".bright_black(), x.name.white().bold())?;
      write!(f, "{x}")?;
    }


    Ok(())
  }
}

