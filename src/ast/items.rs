use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast::{self, TypeId, Visibility}, lexer::Word};


pub struct ModuleItem {
  pub name: String,
  pub ctn: Vec<ast::AnyId>,
}

pub struct GenericItem<'a> {
  pub params: Vec<ast::FieldType<'a>>,
  pub ctn: Vec<ast::AnyId>,
  pub reqs: Vec<(Word<'a>, Vec<TypeId>)>,
}


pub enum ItemVari<'a> {
  Module(ModuleItem),
  Generic(GenericItem<'a>),
  
  Impl{trait_ty: ast::TypeId, type_ty: ast::TypeId, ctn: Vec<ast::DeclId>},

  Import(Vec<(u32, Word<'a>)>, Option<ast::DeclId>),
  ImportWildcard(Vec<(u32, Word<'a>)>, Option<ast::DeclId>),
}

pub struct Item<'a> {
  pub vari: ItemVari<'a>,
  pub vis: Visibility,
}


impl<'a> fmt::Display for Item<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} ", self.vis)?;

    match &self.vari {
      ItemVari::Module(s) => {
        write!(f, "{} {} {}",
          "module".blue().bold(),
          s.name.white().bold(),
          "[".bright_black(),
        )?;

        for (i, x) in s.ctn.iter().enumerate() {
          write!(f, "{}", x)?;
          if i + 1 < s.ctn.len() {
            write!(f, "{}", ", ".bright_black())?;
          }
        }

        write!(f, "{}", "]".bright_black())?;
      }

      ItemVari::Generic(s) => {
        write!(f, "{}", "generic".blue().bold())?;

        if !s.reqs.is_empty() {
          write!(f, " {} ", "requires".yellow().bold())?;

          for (n, ts) in &s.reqs {
            write!(f, "{}{} ", n.str(), ":".bright_black())?;

            for (i, x) in ts.iter().enumerate() {
              write!(f, "{}", x)?;

              if i +1 < ts.len() { write!(f, " {} ", "|".bright_black())?; }
            }

            write!(f, "{} ", ";".bright_black())?;
          }
        }
        
        write!(f, " {}", "[".bright_black())?;
        for (i, x) in s.ctn.iter().enumerate() {
          write!(f, "{}", x)?;

          if i + 1 < s.ctn.len() { write!(f, "{} ", ",".bright_black())?; }
        }
        write!(f, "{}", "]".bright_black())?;
      }
    
      ItemVari::Impl{trait_ty, type_ty, ..} => {
        write!(f, "{} {}{} {}", "impl".blue().bold(), type_ty, ":".bright_black(), trait_ty)?;
      }

      ItemVari::Import(_path, _) => {
        write!(f, "{}", "use".blue().bold())?;
      }
      
      ItemVari::ImportWildcard(_path, _) => {
        write!(f, "{}::*", "use".blue().bold())?;
      }
      
    }
    
    Ok(())
  }
}
