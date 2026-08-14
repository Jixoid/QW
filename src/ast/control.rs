use std::{collections::HashMap, fs, ops::Range, path::Path, ptr};

use crate::{ast::{self, AstId, AstKind}, error::Result};


pub struct Crate<'a> {
  pub name: String,
  pub imod: Vec<&'a Crate<'a>>,

  pub root_exec: ast::ItemId,

  pub str_pool: Vec<String>,

  pub list_type: Vec<ast::Type>,
  pub list_decl: Vec<ast::Decl>,
  pub list_expr: Vec<ast::Expr>,
  pub list_attr: Vec<ast::Attr>,
  pub list_dirc: Vec<ast::Dirc>,
  pub list_item: Vec<ast::Item>,
  pub list_patt: Vec<ast::Patt>,
  pub list_thing: Vec<ast::Thing>,

  pub extra_data: Vec<ast::AnyId>,

  pub map_attr: HashMap<ast::AnyId, Vec<ast::AttrId>>,
}


pub struct Module {
  pub fpath: String,
  pub fid: u16,
  pub name: String,
  pub mmap: Vec<u8>,
}


pub type Rng = Range<u32>;


impl<'a> Crate<'a> {

  pub fn new(name: String) -> Crate<'a> {
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
      list_thing: vec![],

      extra_data: vec![],

      map_attr: HashMap::new(),
    };

    cre
  }


  pub fn add_dep(&mut self, mol: &'a Crate<'a>) -> u16 {
    match self.imod.iter().position(|&x| ptr::eq(x, mol)) {
      Some(r) => (r as u16) +1,
      None => {
        self.imod.push(mol);
        self.imod.len() as u16
      }
    }
  }

  pub fn get_dep(&self, mod_id: u16) -> &'a Crate<'a> {
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


  pub fn localize<T>(&mut self, mol: &'a Crate<'a>, eid: AstId<T>) -> AstId<T> {
    let omol = if eid.krate == 0 { mol } else { mol.get_dep(eid.krate) };

    let lmid = self.add_dep(omol);

    AstId::new(eid.kind, lmid, eid.index)
  }


  pub fn get_type(&self, id: ast::TypeId) -> &ast::Type {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_type[id.index as usize]
  }

  pub fn get_decl(&self, id: ast::DeclId) -> &ast::Decl {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_decl[id.index as usize]
  }

  pub fn get_expr(&self, id: ast::ExprId) -> &ast::Expr {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_expr[id.index as usize]
  }
  
  pub fn get_attr(&self, id: ast::AttrId) -> &ast::Attr {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_attr[id.index as usize]
  }

  pub fn get_dirc(&self, id: ast::DircId) -> &ast::Dirc {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_dirc[id.index as usize]
  }

  pub fn get_item(&self, id: ast::ItemId) -> &ast::Item {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_item[id.index as usize]
  }

  pub fn get_patt(&self, id: ast::PattId) -> &ast::Patt {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_patt[id.index as usize]
  }

  pub fn get_thing(&self, id: ast::ThingId) -> &ast::Thing {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_thing[id.index as usize]
  }

  pub fn get_extra(&self, rng: Rng) -> &[ast::AnyId] {
    let r = (rng.start as usize)..(rng.end as usize);

    &self.extra_data[r]
  }


  pub fn new_type(&mut self, v: ast::Type) -> ast::TypeId {
    self.list_type.push(v);

    let idx = self.list_type.len() as u32 - 1;

    return AstId::new(AstKind::Type, 0, idx);
  }

  pub fn new_decl(&mut self, v: ast::Decl) -> ast::DeclId {
    self.list_decl.push(v);

    let idx = self.list_decl.len() as u32 - 1;

    return AstId::new(AstKind::Decl, 0, idx);
  }

  pub fn new_expr(&mut self, v: ast::Expr) -> ast::ExprId {
    self.list_expr.push(v);

    let idx = self.list_expr.len() as u32 - 1;

    return AstId::new(AstKind::Expr, 0, idx);
  }

  pub fn new_attr(&mut self, v: ast::Attr) -> ast::AttrId {
    self.list_attr.push(v);

    let idx = self.list_attr.len() as u32 - 1;

    return AstId::new(AstKind::Attr, 0, idx);
  }

  pub fn new_dirc(&mut self, v: ast::Dirc) -> ast::DircId {
    self.list_dirc.push(v);

    let idx = self.list_dirc.len() as u32 - 1;

    return AstId::new(AstKind::Dirc, 0, idx);
  }

  pub fn new_item(&mut self, v: ast::Item) -> ast::ItemId {
    self.list_item.push(v);

    let idx = self.list_item.len() as u32 - 1;

    return AstId::new(AstKind::Item, 0, idx);
  }
  
  pub fn new_patt(&mut self, v: ast::Patt) -> ast::PattId {
    self.list_patt.push(v);

    let idx = self.list_patt.len() as u32 - 1;

    return AstId::new(AstKind::Patt, 0, idx);
  }
  
  pub fn new_thing(&mut self, v: ast::Thing) -> ast::ThingId {
    self.list_thing.push(v);

    let idx = self.list_thing.len() as u32 - 1;

    return AstId::new(AstKind::Thing, 0, idx);
  }

  pub fn new_extra(&mut self, v: Vec<ast::AnyId>) -> Rng {
    let start = self.extra_data.len() as u32;

    self.extra_data.extend(v);

    let end = self.extra_data.len() as u32;

    start..end
  }


  pub fn storage_size(&self) -> usize {
    size_of::<ast::Type>()*self.list_type.len()
    +
    size_of::<ast::Decl>()*self.list_decl.len()
    +
    size_of::<ast::Expr>()*self.list_expr.len()
    +
    size_of::<ast::Attr>()*self.list_attr.len()
    +
    size_of::<ast::Dirc>()*self.list_dirc.len()
    +
    size_of::<ast::Item>()*self.list_item.len()
    +
    size_of::<ast::Patt>()*self.list_patt.len()
    +
    size_of::<ast::Thing>()*self.list_thing.len()
    +
    size_of::<ast::AnyId>()*self.extra_data.len()
  }

}


impl Module {

  pub fn new(fpath: &str) -> Result<Module> {
    let mut mmap = fs::read(&fpath)?;
    mmap.resize(mmap.len() + 8, 0);

    let name = String::from(Path::new(&fpath).file_stem().and_then(|s| s.to_str()).unwrap_or(""));

    let mol = ast::Module{
      fpath: fpath.to_string(), name, mmap, fid: 0
    };

    Ok(mol)
  }

}
