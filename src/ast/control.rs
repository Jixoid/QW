use std::{collections::HashMap, fs, ops::Range, path::Path};

use crate::{ast::{self, AstId, AstKind}, error::Result};


pub struct Crate {
  pub root: ast::ItemId,

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


impl<'a> Crate {

  pub fn new() -> Self {
    Self{
      root: AstId::null(),

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
    }
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


  pub fn get_type(&self, id: ast::TypeId) -> &ast::Type {
    assert!(id.kind == ast::AstKind::Type);
    &self.list_type[id.index as usize]
  }

  pub fn get_decl(&self, id: ast::DeclId) -> &ast::Decl {
    assert!(id.kind == ast::AstKind::Decl);
    &self.list_decl[id.index as usize]
  }

  pub fn get_expr(&self, id: ast::ExprId) -> &ast::Expr {
    assert!(id.kind == ast::AstKind::Expr);
    &self.list_expr[id.index as usize]
  }
  
  pub fn get_attr(&self, id: ast::AttrId) -> &ast::Attr {
    assert!(id.kind == ast::AstKind::Attr);
    &self.list_attr[id.index as usize]
  }

  pub fn get_dirc(&self, id: ast::DircId) -> &ast::Dirc {
    assert!(id.kind == ast::AstKind::Dirc);
    &self.list_dirc[id.index as usize]
  }

  pub fn get_item(&self, id: ast::ItemId) -> &ast::Item {
    assert!(id.kind == ast::AstKind::Item);
    &self.list_item[id.index as usize]
  }

  pub fn get_patt(&self, id: ast::PattId) -> &ast::Patt {
    assert!(id.kind == ast::AstKind::Patt);
    &self.list_patt[id.index as usize]
  }

  pub fn get_thing(&self, id: ast::ThingId) -> &ast::Thing {
    assert!(id.kind == ast::AstKind::Thing);
    &self.list_thing[id.index as usize]
  }

  pub fn get_extra(&self, rng: Rng) -> &[ast::AnyId] {
    let r = (rng.start as usize)..(rng.end as usize);

    &self.extra_data[r]
  }


  pub fn new_type(&mut self, v: ast::Type) -> ast::TypeId {
    self.list_type.push(v);

    let idx = self.list_type.len() as u32 - 1;

    return AstId::new(AstKind::Type, idx);
  }

  pub fn new_decl(&mut self, v: ast::Decl) -> ast::DeclId {
    self.list_decl.push(v);

    let idx = self.list_decl.len() as u32 - 1;

    return AstId::new(AstKind::Decl, idx);
  }

  pub fn new_expr(&mut self, v: ast::Expr) -> ast::ExprId {
    self.list_expr.push(v);

    let idx = self.list_expr.len() as u32 - 1;

    return AstId::new(AstKind::Expr, idx);
  }

  pub fn new_attr(&mut self, v: ast::Attr) -> ast::AttrId {
    self.list_attr.push(v);

    let idx = self.list_attr.len() as u32 - 1;

    return AstId::new(AstKind::Attr, idx);
  }

  pub fn new_dirc(&mut self, v: ast::Dirc) -> ast::DircId {
    self.list_dirc.push(v);

    let idx = self.list_dirc.len() as u32 - 1;

    return AstId::new(AstKind::Dirc, idx);
  }

  pub fn new_item(&mut self, v: ast::Item) -> ast::ItemId {
    self.list_item.push(v);

    let idx = self.list_item.len() as u32 - 1;

    return AstId::new(AstKind::Item, idx);
  }
  
  pub fn new_patt(&mut self, v: ast::Patt) -> ast::PattId {
    self.list_patt.push(v);

    let idx = self.list_patt.len() as u32 - 1;

    return AstId::new(AstKind::Patt, idx);
  }
  
  pub fn new_thing(&mut self, v: ast::Thing) -> ast::ThingId {
    self.list_thing.push(v);

    let idx = self.list_thing.len() as u32 - 1;

    return AstId::new(AstKind::Thing, idx);
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
