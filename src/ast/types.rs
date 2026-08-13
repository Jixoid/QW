use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast::{self, ExprId, Rng, TypeId, Visibility}, lexer::Word};


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }


#[derive(Clone)]
pub struct FieldType<'a> {
  pub name: Word<'a>,
  pub kind: TypeId,
  pub vis: Visibility,
  pub attrs: Vec<crate::ast::Attr<'a>>,
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub enum IntegerValue { SIG(i64), USG(u64) }


#[derive(Clone)]
pub struct FieldCons<'a> {
  pub val: IntegerValue,
  pub name: Word<'a>,
}


#[derive(Clone)]
pub struct NickType<'a> {
  pub pos: Word<'a>,
  pub idx: u32
}


pub enum Type<'a> {
  Nick(NickType<'a>),
  Path(Vec<TypeId>),

  InitParam{name: Word<'a>, expr: ExprId},
  
  Ptr   {sub: TypeId, acc: AccessKind},
  Ref   {sub: TypeId, acc: AccessKind},
  Array {sub: TypeId, ext: Vec<u32>},
  Vector{sub: TypeId, ext: u32},
  Range {sub: TypeId},
  Option{sub: TypeId},
  Result{sub: TypeId, err: TypeId},
  
  Struct{vars: Vec<FieldType<'a>>},
  Tuple {vars: Vec<TypeId>},

  Iface{funs: Vec<FieldType<'a>>},
  Trait{funs: Vec<FieldType<'a>>},

  Fun {args: Vec<FieldType<'a>>, ret: Option<TypeId>},
  Init{args: Vec<FieldType<'a>>, ils: Option<Rng>},

  Enum {vals: Vec<FieldCons<'a>>},
  Flags{vals: Vec<FieldCons<'a>>},

  Specialize{base: TypeId, args: Vec<TypeId>},
}

impl<'a> Type<'a> {
  pub fn display<'m>(&'a self, module: &'m ast::Crate) -> TypeDisplay<'a, 'm> {
    TypeDisplay(self, module)
  }
}

pub struct TypeDisplay<'a, 'm>(pub &'a Type<'a>, pub &'m ast::Crate<'m,'m>);

impl<'a, 'm> fmt::Display for TypeDisplay<'a, 'm> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let ty = self.0;
    let mol = self.1;

    match &ty {
      Type::Nick(s) => {
        write!(f, "{}", mol.str_pool[s.idx as usize].yellow().bold())?;
      }

      Type::Path(s) => {
        for (i, x) in s.iter().enumerate() {
          write!(f, "{}", mol.get_type(*x).display(mol))?;
          
          if i +1 < s.len() { write!(f, "{}", "::".bright_black())?; }
        }
      }


      Type::InitParam{name, expr} => {
        write!(f, "{}({})", name.str(), expr)?;
      }


      Type::Ptr{sub, acc} => {
        let acc = match acc { AccessKind::IMM => "imm", AccessKind::MUT => "mut" };

        write!(f, "{}{} {}", "^".blue().bold(), acc.green().bold(), mol.get_type(*sub).display(mol))?;
      }

      Type::Ref{sub, acc} => {
        let acc = match acc { AccessKind::IMM => "imm", AccessKind::MUT => "mut" };

        write!(f, "{}{} {}", "&".blue().bold(), acc.green().bold(), mol.get_type(*sub).display(mol))?;
      }

      Type::Array{sub, ext} => {
        let mut str: String = format!("{}", mol.get_type(*sub).display(mol));
        
        for x in ext { str += &format!("{} {}", ",".bright_black(), x.white().bold()); }

        write!(f, "{}{}{}", "[".bright_black(), str, "]".bright_black())?;
      }

      Type::Vector{sub, ext} => {
        write!(f, "{}{} x {}{}", "[".bright_black(), mol.get_type(*sub).display(mol), ext, "]".bright_black())?;
      }

      Type::Range{sub} => {
        write!(f, "{}{}", "..".bright_black(), mol.get_type(*sub).display(mol))?;
      }

      Type::Option{sub} => {
        write!(f, "{}{}", "?".bright_black(), mol.get_type(*sub).display(mol))?;
      }
      
      Type::Result{sub, err} => {
        write!(f, "{}{}{}", mol.get_type(*sub).display(mol), "?".bright_black(), mol.get_type(*err).display(mol))?;
      }


      Type::Struct{vars} => {
        write!(f, "{}{}", "struct".blue().bold(), "{".bright_black())?;

        for (i, x) in vars.iter().enumerate() {
          write!(f, "{} {}{} {}", x.vis, x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < vars.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", "}".bright_black())?;
      }

      Type::Tuple{vars} => {
        let mut str = String::new();
        
        for (i, x) in vars.iter().enumerate() {
          str += &format!("{}", mol.get_type(*x).display(mol).white().bold());
        
          if i +1 < vars.len() { str += &format!("{}", ", ".bright_black()); }
        }
        
        write!(f, "{}{}{}", "(".bright_black(), str, ")".bright_black())?;
      }
      

      Type::Iface{funs} => {
        write!(f, "{}{}", "iface".blue().bold(), "{".bright_black())?;

        for (i, x) in funs.iter().enumerate() {
          write!(f, "{} {}{} {}", x.vis, x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < funs.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", "}".bright_black())?;
      }

      Type::Trait{funs} => {
        write!(f, "{}{}", "trait".blue().bold(), "{".bright_black())?;

        for (i, x) in funs.iter().enumerate() {
          write!(f, "{} {}{} {}", x.vis, x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < funs.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", "}".bright_black())?;
      }


      Type::Fun{args, ret} => {
        write!(f, "{}{}", "fun".blue().bold(), "(".bright_black())?;
        
        for (i, x) in args.iter().enumerate() {
          write!(f, "{}{} {}", x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < args.len() {
            write!(f, "{} ", ",".bright_black())?;
          }
        }

        write!(f, "{}", ")".bright_black())?;

        if let Some(ret) = ret {
          write!(f, "{}", " -> ".bright_black())?;

          write!(f, "{}", mol.get_type(*ret).display(mol))?;
        }
      }

      Type::Init{args, ils} => {
        write!(f, "{}{}", "init".blue().bold(), "(".bright_black())?;
        
        for (i, x) in args.iter().enumerate() {
          write!(f, "{}{} {}", x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < args.len() {
            write!(f, "{} ", ",".bright_black())?;
          }
        }

        write!(f, "{}", ")".bright_black())?;


        if let Some(ils) = ils {
          write!(f, "{} {:?}", ":".bright_black(), ils)?;
        }
      }


      Type::Enum{vals} => {
        write!(f, "{}{}", "enum".blue().bold(), "{".bright_black())?;
        for (i, x) in vals.iter().enumerate() {
          write!(f, "{}", x.name.str().blue().bold())?;
          let val_str = match x.val {
            IntegerValue::SIG(v) => format!("{}", v),
            IntegerValue::USG(v) => format!("{}", v),
          };
          write!(f, " {} {}", "=".bright_black(), val_str.green())?;
          if i + 1 < vals.len() {
            write!(f, "{} ", ",".bright_black())?;
          }
        }
        write!(f, "{}", "}".bright_black())?;
      }

      Type::Flags{vals} => {
        write!(f, "{}{}", "flags".blue().bold(), "{".bright_black())?;
        for (i, x) in vals.iter().enumerate() {
          write!(f, "{}", x.name.str().blue().bold())?;
          let val_str = match x.val {
            IntegerValue::SIG(v) => format!("{}", v),
            IntegerValue::USG(v) => format!("{}", v),
          };
          write!(f, " {} {}", "=".bright_black(), val_str.green())?;
          if i + 1 < vals.len() {
            write!(f, "{} ", ",".bright_black())?;
          }
        }
        write!(f, "{}", "}".bright_black())?;
      }


      Type::Specialize{base, args} => {
        write!(f, "{} {}<", "generic".blue().bold(), mol.get_type(*base).display(mol))?;
        
        for (i, x) in args.iter().enumerate() {
          write!(f, "{}", mol.get_type(*x).display(mol))?;
          
          if i+1 < args.len() { write!(f, ", ")? }
        }

        write!(f, ">")?;
      }

    }

    Ok(())
  }
}
