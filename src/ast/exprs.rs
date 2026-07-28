use core::fmt;
use owo_colors::OwoColorize;

use crate::{control::{IdentyId, Module}, lexer::{Word, WordKind}};



#[derive(Debug)]
pub struct BlockExpr<'a> {
  pub label: Option<Word<'a>>,
  pub ctn: Vec<IdentyId>,
}

#[derive(Debug)]
pub struct IfExpr {
  pub cond: IdentyId,
  pub then_block: IdentyId,
  pub else_block: Option<IdentyId>,
}

#[derive(Debug)]
pub struct MatchArm {
  pub pat: IdentyId,
  pub body: IdentyId,
}

#[derive(Debug)]
pub struct MatchExpr {
  pub val: IdentyId,
  pub arms: Vec<MatchArm>,
}


#[derive(Debug)]
pub struct NickExpr<'a> {
  pub pos: Word<'a>,
  pub idx: u32,
  pub resolved: Option<IdentyId>,
}

impl<'a> NickExpr<'a> {
  pub fn new(mol: &mut Module, p: Word<'a>) -> NickExpr<'a> {
    let idx = match mol.nick_map.iter().position(|x| x == p.str()) {
      Some(r) => r,
      None => {
        let a = mol.nick_map.len();
        mol.nick_map.push(p.string());
        a
      }
    } as u32;

    NickExpr{pos: p, idx, resolved: None}
  }
}


#[derive(Debug)]
pub struct BinaryExpr {
  pub lhs: IdentyId,
  pub rhs: IdentyId,
  pub op: WordKind,
}

#[derive(Debug)]
pub struct UnaryExpr {
  pub op: WordKind,
  pub val: IdentyId,
}


#[derive(Debug)]
pub struct NumberExpr<'a> {
  pub pos: Word<'a>,
}


#[derive(Debug)]
pub enum ExprVari<'a> {
  Block(BlockExpr<'a>),
  If(IfExpr),
  Match(MatchExpr),
  Nick(NickExpr<'a>),
  Binary(BinaryExpr),
  Unary(UnaryExpr),
  Number(NumberExpr<'a>),
}

#[derive(Debug)]
pub struct Expr<'a> {
  pub vari: ExprVari<'a>,
  pub ty: IdentyId,
}

impl<'a> Expr<'a> {
  pub fn display<'m>(&'a self, module: &'m Module<'m,'m>) -> ExprDisplay<'a, 'm> {
    ExprDisplay(self, module)
  }

  pub fn vari_is_if(&self) -> bool {
    matches!(self.vari, ExprVari::If(_) | ExprVari::Match(_))
  }
}


pub struct ExprDisplay<'a,'m>(pub &'a Expr<'a>, pub &'m Module<'m,'m>);

impl<'a,'m> fmt::Display for ExprDisplay<'a,'m> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let expr = self.0;
    let mol = self.1;

    match &expr.vari {
      ExprVari::Block(s) => {
        write!(f, "{}", "block".blue().bold())?;
        
        if let Some(lbl) = s.label {
          write!(f, " {}{}", lbl.str().white().bold(), ":".bright_black())?;
        }

        write!(f, "{}", " [".bright_black())?;

        for (i, x) in s.ctn.iter().enumerate() {
          write!(f, "{}", x)?;
          if i + 1 < s.ctn.len() {
            write!(f, "{}", ", ".bright_black())?;
          }
        }

        write!(f, "{}", "]".bright_black())?;
      }
      ExprVari::If(s) => {
        write!(f, "{} {} {} {}",
          "if".blue().bold(),
          s.cond,
          "then".blue().bold(),
          s.then_block
        )?;

        if let Some(eb) = s.else_block {
          if mol.get_expr(eb).vari_is_if() {
            write!(f, " {} {}", "ef".blue().bold(), eb)?;
          } else {
            write!(f, " {} {}", "else".blue().bold(), eb)?;
          }
        }
      }
      ExprVari::Match(s) => {
        write!(f, "{} {} {}",
          "match".blue().bold(),
          s.val,
          "{".bright_black()
        )?;
        
        for arm in &s.arms {
          write!(f, " {} {} {},", 
            arm.pat,
            "=>".bright_black(),
            arm.body
          )?;
        }
        write!(f, " {}", "}".bright_black())?;
      }
      ExprVari::Nick(s) => { 
        write!(f, "{}{}{}{:x}",
          "\"".yellow().bold(),
          mol.nick_map[s.idx as usize].yellow().bold(),
          "\"".yellow().bold(),
          s.idx
        )?;
      }
      ExprVari::Binary(s) => {
        write!(f, "{} {} {:?} {}",
          "binary".blue().bold(),
          s.lhs,
          s.op,
          s.rhs,
        )?;
      }
      ExprVari::Unary(s) => {
        write!(f, "{} {:?} {}",
          "unary".blue().bold(),
          s.op,
          s.val,
        )?;
      }
      ExprVari::Number(s) => {
        write!(f, "{}", s.pos.str().yellow().bold())?;
      }
    };
    
    Ok(())
  }
}
