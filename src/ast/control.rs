use core::fmt;
use owo_colors::OwoColorize;
use std::{collections::HashMap, fs, ops::Range, path::Path, ptr};

use crate::{ast::{self, AstId, AstKind}, error::Result};


pub struct Crate<'a, 'd:'a> {
  pub name: String,
  pub imod: Vec<&'d Crate<'d,'d>>,

  pub root_exec: ast::ItemId,

  pub str_pool: Vec<String>,

  pub list_type: Vec<ast::Type<'a>>,
  pub list_decl: Vec<ast::Decl<'a>>,
  pub list_expr: Vec<ast::Expr<'a>>,
  pub list_attr: Vec<ast::Attr<'a>>,
  pub list_dirc: Vec<ast::Dirc<'a>>,
  pub list_item: Vec<ast::Item<'a>>,
  pub list_patt: Vec<ast::Patt<'a>>,

  pub extra_data: Vec<ast::AnyId>,

  pub map_attr: HashMap<ast::AnyId, Vec<ast::AttrId>>,
}


pub struct Module {
  pub fpath: String,
  pub name: String,
  pub mmap: Vec<u8>,
}


pub type Rng = Range<u32>;


impl<'a,'d> Crate<'a,'d> {

  pub fn new(name: String) -> Crate<'a,'d> {
    let cre = Crate{
      name,
      imod: vec![],
      root_exec: AstId::null(),
      str_pool: vec![],

      list_type: vec![],
      list_decl: vec![],
      list_expr: vec![],
      list_attr: vec![],
      list_dirc: vec![],
      list_item: vec![],
      list_patt: vec![],

      extra_data: vec![],

      map_attr: HashMap::new(),
    };

    cre
  }


  pub fn add_dep(&mut self, mol: &'d Crate<'d,'d>) -> u16 {
    match self.imod.iter().position(|&x| ptr::eq(x, mol)) {
      Some(r) => (r as u16) +1,
      None => {
        self.imod.push(mol);
        self.imod.len() as u16
      }
    }
  }

  pub fn get_dep(&self, mod_id: u16) -> &'d Crate<'d,'d> {
    self.imod[(mod_id -1) as usize] 
  }


  pub fn get_str(&mut self, s: &str) -> u32 {
    let idx = match self.str_pool.iter().position(|x| x == s) {
      Some(r) => r,
      None => {
        let a = self.str_pool.len();
        self.str_pool.push(s.to_string());
        a
      }
    } as u32;

    idx
  }


  pub fn localize<T>(&mut self, mol: &'d Crate<'d,'d>, eid: AstId<T>) -> AstId<T> {
    let omol: &Crate<'d,'d> = if eid.krate == 0 { mol } else { mol.get_dep(eid.krate) };

    let lmid = self.add_dep(omol);

    AstId::new(eid.kind, lmid, eid.index)
  }


  pub fn get_type(&self, id: ast::TypeId) -> &ast::Type<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_type[id.index as usize]
  }

  pub fn get_decl(&self, id: ast::DeclId) -> &ast::Decl<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_decl[id.index as usize]
  }

  pub fn get_expr(&self, id: ast::ExprId) -> &ast::Expr<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_expr[id.index as usize]
  }
  
  pub fn get_attr(&self, id: ast::AttrId) -> &ast::Attr<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_attr[id.index as usize]
  }

  pub fn get_dirc(&self, id: ast::DircId) -> &ast::Dirc<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_dirc[id.index as usize]
  }

  pub fn get_item(&self, id: ast::ItemId) -> &ast::Item<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_item[id.index as usize]
  }

  pub fn get_patt(&self, id: ast::PattId) -> &ast::Patt<'a> {
    let modl: &Crate<'a,'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_patt[id.index as usize]
  }

  pub fn get_extra(&self, rng: Rng) -> &[ast::AnyId] {
    let r = (rng.start as usize)..(rng.end as usize);

    &self.extra_data[r]
  }


  pub fn new_type(&mut self, v: ast::Type<'a>) -> ast::TypeId {
    self.list_type.push(v);

    let idx = self.list_type.len() as u32 - 1;

    return AstId::new(AstKind::Type, 0, idx);
  }

  pub fn new_decl(&mut self, v: ast::Decl<'a>) -> ast::DeclId {
    self.list_decl.push(v);

    let idx = self.list_decl.len() as u32 - 1;

    return AstId::new(AstKind::Decl, 0, idx);
  }

  pub fn new_expr(&mut self, v: ast::Expr<'a>) -> ast::ExprId {
    self.list_expr.push(v);

    let idx = self.list_expr.len() as u32 - 1;

    return AstId::new(AstKind::Expr, 0, idx);
  }

  pub fn new_attr(&mut self, v: ast::Attr<'a>) -> ast::AttrId {
    self.list_attr.push(v);

    let idx = self.list_attr.len() as u32 - 1;

    return AstId::new(AstKind::Attr, 0, idx);
  }

  pub fn new_dirc(&mut self, v: ast::Dirc<'a>) -> ast::DircId {
    self.list_dirc.push(v);

    let idx = self.list_dirc.len() as u32 - 1;

    return AstId::new(AstKind::Dirc, 0, idx);
  }

  pub fn new_item(&mut self, v: ast::Item<'a>) -> ast::ItemId {
    self.list_item.push(v);

    let idx = self.list_item.len() as u32 - 1;

    return AstId::new(AstKind::Item, 0, idx);
  }
  
  pub fn new_patt(&mut self, v: ast::Patt<'a>) -> ast::PattId {
    self.list_patt.push(v);

    let idx = self.list_patt.len() as u32 - 1;

    return AstId::new(AstKind::Patt, 0, idx);
  }

  pub fn new_extra(&mut self, v: Vec<ast::AnyId>) -> Rng {
    let start = self.extra_data.len() as u32;

    self.extra_data.extend(v);

    let end = self.extra_data.len() as u32;

    start..end
  }


  pub fn storage_size(&self) -> usize {

    let type_size = size_of::<ast::Type>()*self.list_type.len();
    let decl_size = size_of::<ast::Decl>()*self.list_decl.len();
    let expr_size = size_of::<ast::Expr>()*self.list_expr.len();
    let attr_size = size_of::<ast::Attr>()*self.list_attr.len();
    let dirc_size = size_of::<ast::Dirc>()*self.list_dirc.len();
    let item_size = size_of::<ast::Item>()*self.list_item.len();
    let patt_size = size_of::<ast::Patt>()*self.list_patt.len();
    let extra_size = size_of::<ast::AnyId>()*self.extra_data.len();

    type_size+decl_size+expr_size+attr_size+dirc_size+item_size+patt_size+extra_size
  }

}


impl<'a,'d> fmt::Display for Crate<'a,'d> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  {}{}", "inter-module".purple().bold(), ":".bright_black())?;
    for (i, x) in self.imod.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i+1, "]".bright_black(), x.name)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "str-pool".purple().bold(), ":".bright_black())?;
    for (i, x) in self.str_pool.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "types".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_type.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.display(self))?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "decls".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_decl.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "exprs".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_expr.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.display(self))?;
    }
    writeln!(f)?;

    
    writeln!(f, "  {}{}", "attrs".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_attr.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "dircs".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_dirc.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "items".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_item.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x)?;
    }
    writeln!(f)?;


    writeln!(f, "  {}{}", "patt".purple().bold(), ":".bright_black())?;
    for (i, x) in self.list_patt.iter().enumerate() {
      writeln!(f, "    {}{:x}{} {}", "[".bright_black(), i, "]".bright_black(), x.display(self))?;
    }
    writeln!(f)?;

    Ok(())
  }
}



impl Module {

  pub fn new(fpath: &str) -> Result<Module> {
    let mut mmap = fs::read(&fpath)?;
    mmap.resize(mmap.len() + 8, 0);

    let name = String::from(Path::new(&fpath).file_stem().and_then(|s| s.to_str()).unwrap_or(""));

    let mol = ast::Module{
      fpath: fpath.to_string(), name, mmap
    };

    Ok(mol)
  }

}
