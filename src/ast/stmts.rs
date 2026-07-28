use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast::AccessKind, control::{IdentyId, Module}, lexer::Word};


#[derive(Debug)]
pub struct LetStmt<'a> {
  pub name: Word<'a>,
  pub kind: IdentyId,
  pub init: Option<IdentyId>,
  pub acck: AccessKind,
}

#[derive(Debug)]
pub struct RetStmt {
  pub val: IdentyId,
}

#[derive(Debug)]
pub struct ExprStmt {
  pub expr: IdentyId,
}

#[derive(Debug)]
pub enum StmtVari<'a> {
  Let(LetStmt<'a>),
  Ret(RetStmt),
  Expr(ExprStmt),
}


#[derive(Debug)]
pub struct Stmt<'a> {
  pub vari: StmtVari<'a>,
}

impl<'a> Stmt<'a> {
  pub fn display<'m>(&'a self, module: &'m Module<'m,'m>) -> StmtDisplay<'a, 'm> {
    StmtDisplay(self, module)
  }
}


pub struct StmtDisplay<'a, 'm>(pub &'a Stmt<'a>, pub &'m Module<'m,'m>);

impl<'a, 'm> fmt::Display for StmtDisplay<'a, 'm> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let stmt = self.0;
    let mol = self.1;

    match &stmt.vari {
      StmtVari::Let(s) => {
        write!(f, "{} {} {}",
          "let".blue().bold(),
          match s.acck {
            AccessKind::IMM => "imm",
            AccessKind::MUT => "mut",
          }.green().bold(),
          s.name.str().white().bold()
        )?;

        if s.kind.kind() != crate::control::IdentyKind::Null {
          write!(f, "{} {}", ":".bright_black(), s.kind)?;
        }

        if let Some(init) = s.init {
          write!(f, " {} {}", "=".bright_black(), init)?;
        }

        write!(f, "{}", ";".bright_black())?;
      }
      StmtVari::Ret(s) => {
        write!(f, "{} {}{}",
          "ret".blue().bold(),
          s.val,
          ";".bright_black(),
        )?;
      }
      StmtVari::Expr(s) => {
        write!(f, "{}{}",
          mol.get_expr(s.expr).display(mol),
          ";".bright_black(),
        )?;
      }
    };
    
    Ok(())
  }
}
