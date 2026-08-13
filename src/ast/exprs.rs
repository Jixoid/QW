use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast::{self, AccessKind, Crate, ExprId, PattId, Rng, TypeId}, lexer::Word};



pub struct MatchArm {
  pub pat: ast::ExprId,
  pub body: ast::ExprId,
}

#[derive(Debug)]
pub enum BinaryOp {
  Add, Sub,
  Mul, Div, Mod,
  Eq, Neq,
  Lt, Gt,
  Lte, Gte,
  And, Or,
  Assign,
}

#[derive(Debug)]
pub enum UnaryOp {
  Neg, Poz,
  Not,
  Deref,
  Ref,
  BitNot,
}


pub enum NumberConst {
  I64(i64),
  U64(u64),
  F64(f64),
}

pub struct NumberExpr<'a> {
  pub pos: Word<'a>,
  pub num: NumberConst,
}

pub enum Expr<'a> {
  Nick{pos: Word<'a>, idx: u32},
  Path(Vec<ExprId>),
  Member(Vec<ExprId>),

  Tuple(Vec<ExprId>),

  Number(NumberExpr<'a>),
  String(Word<'a>),

  Block{label: Option<Word<'a>>, rng: Rng, expr: Option<ExprId>},

  If   {cond: ExprId, then: ExprId, elsb: Option<ExprId>},
  Match{cond: ExprId, arms: Vec<MatchArm>},
  While{cond: ExprId, blok: ExprId, elsb: Option<ExprId>},
  Loop {blok: ExprId, elsb: Option<ExprId>},
  ForIn{vars: PattId, iter: ExprId, blok: ExprId, elsb: Option<ExprId>},
  
  Binary{op: BinaryOp, lhs: ExprId, rhs: ExprId},
  Unary {op: UnaryOp, val: ExprId},
  
  Call {callee: ExprId, args: Vec<ExprId>},
  Index{callee: ExprId, args: Vec<ExprId>},

  Let{item: PattId, kind: Option<TypeId>, init: Option<ExprId>, acck: AccessKind},

  Return  {label: Option<Word<'a>>, val: Option<ExprId>},
  Break   {label: Option<Word<'a>>, val: Option<ExprId>},
  Continue{label: Option<Word<'a>>},

  Try(ExprId),
  Unwrap(ExprId),
}


impl<'a> Expr<'a> {
  pub fn display<'m>(&'a self, module: &'m Crate<'m,'m>) -> ExprDisplay<'a, 'm> {
    ExprDisplay(self, module)
  }

  pub fn vari_is_if(&self) -> bool { matches!(self, Expr::If{..} | Expr::Match{..}) }
}


pub struct ExprDisplay<'a,'m>(pub &'a Expr<'a>, pub &'m Crate<'m,'m>);

impl<'a,'m> fmt::Display for ExprDisplay<'a,'m> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let expr = self.0;
    let mol = self.1;

    match &expr {
      Expr::Nick{pos, ..} => {
        write!(f, "{}", pos.str())?;
      }

      Expr::Path(p) => {
        for (i, x) in p.iter().enumerate() {
          write!(f, "{}", mol.get_expr(*x).display(mol))?;
          
          if i + 1 < p.len() { write!(f, "{}", "::".bright_black())?; }
        }
      }

      Expr::Member(p) => {
        for (i, x) in p.iter().enumerate() {
          write!(f, "{}", mol.get_expr(*x).display(mol))?;
          
          if i + 1 < p.len() { write!(f, "{}", ".".bright_black())?; }
        }
      }


      Expr::Tuple(s) => {
        write!(f, "{}", "(".bright_black())?;

        for (i, x) in s.iter().enumerate() {
          write!(f, "{}", mol.get_expr(*x).display(mol))?;

          if i +1 < s.len() { write!(f, "{}", ", ".bright_black())?; }
        }

        write!(f, "{}", ")".bright_black())?;
      }
      
      
      Expr::Number(s) => {
        write!(f, "{}", s.pos.str().yellow().bold())?;
      }
      
      Expr::String(s) => {
        write!(f, "\"{}\"", s.str().yellow().bold())?;
      }
      

      Expr::Block{label, rng, expr} => {
        write!(f, "{}", "block".blue().bold())?;
        
        if let Some(lbl) = label {
          write!(f, " {}{}", lbl.str().white().bold(), ":".bright_black())?;
        }

        write!(f, "{}{:?}", " [".bright_black(), rng)?;

        if let Some(e) = expr {
          write!(f, "{} {}", "ret".yellow().bold(), e)?;
        }

        write!(f, "{}", "]".bright_black())?;
      }
      
      
      Expr::If{cond, then, elsb} => {
        write!(f, "{} {} {} {}", "if".blue().bold(), cond, "then".blue().bold(), then)?;

        if let Some(eb) = elsb {
          if mol.get_expr(*eb).vari_is_if() {
            write!(f, " {} {}", "ef".blue().bold(), eb)?;
          } else {
            write!(f, " {} {}", "else".blue().bold(), eb)?;
          }
        }
      }

      Expr::Match{cond, arms} => {
        write!(f, "{} {} {}", "match".blue().bold(), cond, "{".bright_black())?;
        
        for arm in arms {
          write!(f, " {} {} {},", 
            arm.pat,
            "=>".bright_black(),
            arm.body
          )?;
        }
        write!(f, " {}", "}".bright_black())?;
      }

      Expr::While{cond, blok, elsb} => {
        write!(f, "{} {} {}", "while".blue().bold(), cond, blok)?;

        if let Some(elsb) = elsb {
          write!(f, " {} {}", "else".blue().bold(), elsb)?;
        }
      }

      Expr::Loop{blok, elsb} => {
        write!(f, "{} {}", "loop".blue().bold(), blok)?;

        if let Some(elsb) = elsb {
          write!(f, " {} {}", "else".blue().bold(), elsb)?;
        }
      }

      Expr::ForIn{vars, iter, blok, elsb} => {
        write!(f, "{} {} {} {} {}",
          "for".blue().bold(),
          vars,
          "in".blue().bold(),
          iter,
          blok
        )?;

        if let Some(elsb) = elsb {
          write!(f, " {} {}", "else".blue().bold(), elsb)?;
        }
      }


      Expr::Binary{op, lhs, rhs} => {
        write!(f, "{} {} {:?} {}", "binary".blue().bold(), lhs, op, rhs)?;
      }
      
      Expr::Unary{op, val} => {
        write!(f, "{} {:?} {}", "unary".blue().bold(), op, val )?;
      }
      

      Expr::Call{callee, args} => {
        write!(f, "{} {}(", "call".blue().bold(), callee)?;
        for (i, arg) in args.iter().enumerate() {
          write!(f, "{}", arg)?;
          
          if i + 1 < args.len() { write!(f, ", ")?; }
        }
        write!(f, ")")?;
      }
      
      Expr::Index{callee, args} => {
        write!(f, "{} {}[", "index".blue().bold(), callee)?;
        for (i, arg) in args.iter().enumerate() {
          write!(f, "{}", arg)?;
          
          if i + 1 < args.len() { write!(f, ", ")?; }
        }
        write!(f, "]")?;
      }
    

      Expr::Let{item, kind, init, acck} => {
        write!(f, "{} {} {}",
          "let".blue().bold(),
          match acck {
            AccessKind::IMM => "imm",
            AccessKind::MUT => "mut",
          }.green().bold(),
          
          item
        )?;

        if let Some(kind) = kind {
          write!(f, "{} {}", ":".bright_black(), kind)?;
        }

        if let Some(init) = init {
          write!(f, " {} {}", "=".bright_black(), init)?;
        }
      }


      Expr::Return{label, val} => {
        write!(f, "{}", "ret".blue().bold())?;

        if let Some(lab) = label {
          write!(f, " `{}", lab.str())?;
        }

        if let Some(val) = val {
          write!(f, " {}", mol.get_expr(*val).display(mol))?;
        }
      }

      Expr::Break{label, val} => {
        write!(f, "{}", "break".blue().bold())?;

        if let Some(lab) = label {
          write!(f, " `{}", lab.str())?;
        }

        if let Some(val) = val {
          write!(f, " {}", mol.get_expr(*val).display(mol))?;
        }
      }

      Expr::Continue{label} => {
        write!(f, "{}", "continue".blue().bold())?;

        if let Some(lab) = label {
          write!(f, " `{}", lab.str())?;
        }
      }


      Expr::Try(s) => {
        write!(f, "{}?", mol.get_expr(*s).display(mol))?;
      }

      Expr::Unwrap(s) => {
        write!(f, "{}!", mol.get_expr(*s).display(mol))?;
      }

    };
    
    Ok(())
  }
}
