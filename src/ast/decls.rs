use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast, lexer::Word};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Visibility {
  Public,
  Private,
  Protected,
  Crate,
  Group,
}

impl fmt::Display for Visibility {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let k = match self {
      Visibility::Public => "pub",
      Visibility::Private => "priv",
      Visibility::Protected => "prot",
      Visibility::Crate => "crate",
      Visibility::Group => "group",
    };

    write!(f, "{}", k.green().bold())?;
    Ok(())
  }
}


pub struct VarDecl {
  pub kind: ast::TypeId,
  pub comptime: bool,
  pub init: Option<ast::ExprId>,
  pub acck: ast::AccessKind,
}

pub struct FunDecl {
  pub kind: ast::TypeId,
  pub blok: ast::ExprId,
}


pub enum DeclVari {
  Var(VarDecl),
  Fun(FunDecl),
  Using(ast::TypeId),
}


pub struct Decl<'a> {
  pub name: Word<'a>,
  pub vari: DeclVari,
  pub vis: Visibility,
}

impl<'a> Decl<'a> {

  pub fn new(name: Word<'a>, vari: DeclVari, vis: Visibility) -> Decl<'a> {
    Decl {name, vari, vis}
  }
  
}


impl<'a> fmt::Display for Decl<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} ", self.vis)?;
    
    match &self.vari {
      DeclVari::Var(_s) => {
        write!(f, "{} {}", "var".blue().bold(), self.name.str().white().bold())?;
      }
      
      DeclVari::Fun(s) => {
        write!(f, "{} {}{} {} {} {}",
          "fun".blue().bold(),
          self.name.str().white().bold(),
          ":".bright_black(),
          s.kind,
          "=".bright_black(),
          s.blok,
        )?;
      }
      
      DeclVari::Using(s) => {
        write!(f, "{} {} {} {}",
          "using".blue().bold(),
          self.name.str().white().bold(),
          "=".bright_black(),
          s
        )?;
      }
      
    };
    
    Ok(())
  }
}
