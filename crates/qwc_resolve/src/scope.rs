use qwc_ast::{self as ast, Ident};
use qwc_hir as hir;
use qwc_string_interner::Sid;
use rustc_hash::FxHashMap;
use qwc_diagnostic::{Label, Message, Span, Summary, msg::*};


pub struct ScopeMap {
  pub(crate) map: FxHashMap<ast::AnyId, Scope>,
}

impl ScopeMap {

  pub fn new() -> Self {
    Self {
      map: FxHashMap::default(),
    }
  }

  pub fn get(&self, id: &ast::AnyId) -> Option<&Scope> {
    self.map.get(id)
  }

  pub fn get_mut(&mut self, id: &ast::AnyId) -> Option<&mut Scope> {
    self.map.get_mut(id)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&ast::AnyId, &Scope)> {
    self.map.iter()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ast::AnyId, &mut Scope)> {
    self.map.iter_mut()
  }

}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub enum ScopeKind {
  Ast(ScopeKindAst),
  Hir(ScopeKindHir),
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKindAst {
  Type(ast::TypeId),
  TypeParam(ast::ThingId),
  
  Expr(ast::ItemId),
  ExprParam(ast::ThingId),
  
  Module(ast::ItemId),
  Local(u32),
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKindHir {
  Type(hir::TypeId),
  
  Expr(hir::ItemId, hir::TypeId),
  
  Module(hir::ItemId),  
}




#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImportSegment {
  Crate,
  Super,
  Name(Ident),
}

impl ImportSegment {
  pub fn as_ident(&self) -> Option<Ident> {
    match self {
      Self::Name(ident) => Some(*ident),
      _ => None,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportStatus {
  Unsolved,
  Solved,
  Failed,
}

#[derive(Clone, Debug)]
pub struct ImportDef {
  pub span: Span,
  pub vis: qwc_ast::Visibility,
  pub segments: Vec<ImportSegment>,
  pub glob: bool,
  pub status: ImportStatus,
}

impl ImportDef {
  pub fn new(span: Span, vis: qwc_ast::Visibility, segments: Vec<ImportSegment>, glob: bool) -> Self {
    Self {
      span,
      vis,
      segments,
      glob,
      status: ImportStatus::Unsolved,
    }
  }

  pub fn is_solved(&self) -> bool {
    matches!(self.status, ImportStatus::Solved)
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertResult {
  Inserted,
  AlreadyPresentSame,
  Conflict(Span),
}



pub struct Scope {
  pub parent: Option<ast::AnyId>,
  pub import: Vec<ImportDef>,
  pub(crate) map: FxHashMap<Sid, (ScopeKind, Span)>,
}

impl Scope {

  pub fn new(parent: Option<ast::AnyId>) -> Self {
    Self {
      parent,
      import: vec![],
      map: FxHashMap::default(),
    }
  }

  pub fn insert(&mut self, ident: Ident, kind: ScopeKindAst, sum: &mut Summary) {
    use std::collections::hash_map::Entry;

    match self.map.entry(ident.sid()) {
      Entry::Occupied(entry) => {
        let (_, last) = entry.get();
        sum.add(Message::error(DUPLICATE_IDENTIFIER, Label::new(ident, CONFLICTING_DEFINITION)).add(Label::new(*last, FIRST_DEFINITION_HERE)));
      }
      Entry::Vacant(entry) => {
        entry.insert((ScopeKind::Ast(kind), ident.into()));
      }
    }
  }

  pub fn insert_import(&mut self, sid: Sid, kind: ScopeKind, span: Span) -> InsertResult {
    use std::collections::hash_map::Entry;

    match self.map.entry(sid) {
      Entry::Occupied(entry) => {
        let &(existing_kind, existing_span) = entry.get();
        if existing_kind == kind {
          InsertResult::AlreadyPresentSame
        } else {
          InsertResult::Conflict(existing_span)
        }
      }
      Entry::Vacant(entry) => {
        entry.insert((kind, span));
        InsertResult::Inserted
      }
    }
  }

  pub fn get(&self, sid: &Sid) -> Option<&(ScopeKind, Span)> {
    self.map.get(sid)
  }

  pub fn iter(&self) -> impl Iterator<Item = (&Sid, &(ScopeKind, Span))> {
    self.map.iter()
  }

  pub fn contains_key(&self, sid: &Sid) -> bool {
    self.map.contains_key(sid)
  }

}
