use core::fmt;
use owo_colors::OwoColorize;

use crate::{control::identy::IdentyId, lexer::Word};


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


#[derive(Debug)]
pub struct ModuleDecl {
  pub decls: Vec<IdentyId>,
}

#[derive(Debug)]
pub struct VarDecl {
  pub kind: IdentyId,
  pub comptime: bool,
  pub init: Option<IdentyId>,
  pub acck: crate::ast::types::AccessKind,
}

#[derive(Debug)]
pub struct FunDecl {
  pub kind: IdentyId,
  pub blok: IdentyId,
}


#[derive(Debug)]
pub enum DeclVari {
  Module(ModuleDecl),
  Var(VarDecl),
  Fun(FunDecl),
  Using(IdentyId),
}


#[derive(Debug)]
pub enum DeclName<'a> {
  Name(String),
  Word(Word<'a>)
}

impl<'a> fmt::Display for DeclName<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self {
      DeclName::Name(s) => write!(f, "{}", s),
      DeclName::Word(s) => write!(f, "{}", s.str()),
    }
  }
}


#[derive(Debug)]
pub struct Decl<'a> {
  pub name: DeclName<'a>,
  pub vari: DeclVari,
  pub vis: Visibility,
}

impl<'a> Decl<'a> {

  pub fn new(name: Word<'a>, vari: DeclVari, vis: Visibility) -> Decl<'a> { Decl{name: DeclName::Word(name), vari, vis} }
  
  pub fn new_str(name: String, vari: DeclVari, vis: Visibility) -> Decl<'a> { Decl{name: DeclName::Name(name), vari, vis} }

}


impl<'a> fmt::Display for Decl<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.vari {
      DeclVari::Module(s) => {
        write!(f, "{} {} {} {}",
          self.vis,
          "module".blue().bold(),
          self.name.white().bold(),
          "[".bright_black(),
        )?;

        for (i, x) in s.decls.iter().enumerate() {
          write!(f, "{}", x)?;
          if i + 1 < s.decls.len() {
            write!(f, "{}", ", ".bright_black())?;
          }
        }

        write!(f, "{}", "]".bright_black())?;
      }
      DeclVari::Var(_s)       => write!(f, "{} {} {}", self.vis, "var".blue().bold(), self.name.white().bold())?,
      DeclVari::Fun(s)   => {
        write!(f, "{} {} {}{} {} {} {}",
          self.vis,
          "fun".blue().bold(),
          self.name.white().bold(),
          ":".bright_black(),
          s.kind,
          "=".bright_black(),
          s.blok,
        )?;
      }
      DeclVari::Using(s)    => {
        write!(f, "{} {} {} {} {}",
          self.vis,
          "using".blue().bold(),
          self.name.white().bold(),
          "=".bright_black(),
          s
        )?;
      }
    };
    
    Ok(())
  }
}
