use core::fmt;
use owo_colors::OwoColorize;

use crate::{ast::Visibility, control::{identy::IdentyId, module::Module}, lexer::Word};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind { IMM, MUT }


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldType<'a> {
  pub name: Word<'a>,
  pub kind: IdentyId,
  pub vis: Visibility,
  pub attrs: Vec<crate::ast::Attribute<'a>>,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IntegerValue {
  SIG(i64),
  USG(u64),
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldCons<'a> {
  pub val: IntegerValue,
  pub name: Word<'a>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NickType<'a> {
  pub pos: Word<'a>,
  pub idx: u32
}

impl<'a> NickType<'a> {

  pub fn new(mol: &mut Module, p: Word<'a>) -> NickType<'a> {
    let idx = match mol.nick_map.iter().position(|x| x == p.str()) {
      Some(r) => r,
      None => {
        let a = mol.nick_map.len();
        mol.nick_map.push(p.string());
        a
      }
    } as u32;

    NickType{pos: p, idx}
  }

}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PArrayType {
  pub sub: IdentyId,
  pub size: u64,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunType<'a> {
  pub args: Vec<FieldType<'a>>,
  pub is_static: bool,
  pub is_const: bool,
  pub ret: IdentyId,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType<'a> {
  pub base: Vec<IdentyId>,
  pub vars: Vec<FieldType<'a>>,
  pub funs: Vec<IdentyId>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumType<'a> {
  pub vals: Vec<FieldCons<'a>>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagsType<'a> {
  pub vals: Vec<FieldCons<'a>>,
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfaceType<'a> {
  pub funs: Vec<FieldType<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitType<'a> {
  pub funs: Vec<FieldType<'a>>,
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeVari<'a> {
  Int{bit: u32, sig: bool},
  Float{bit: u32},
  ArchSize{sig: bool},
  Bool,
  Char,
  Ptr,
  Void,
  Null,

  Nick(NickType<'a>),
  SelfType,
  UnresolvedPath(Vec<NickType<'a>>),
  Path(Vec<IdentyId>),
  
  PointerOf{sub: IdentyId, acc: AccessKind},
  ReferenceOf{sub: IdentyId, acc: AccessKind},
  ZArrayOf(IdentyId),
  PArrayOf(PArrayType),

  Function(FunType<'a>),
  Struct(StructType<'a>),
  Iface(IfaceType<'a>),
  Trait(TraitType<'a>),
  Enum(EnumType<'a>),
  Flags(FlagsType<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeState {
  Unresolved,
  Resolving,
  Resolved,
}

#[derive(Debug)]
pub struct Type<'a> {
  pub vari: TypeVari<'a>,
  pub state: TypeState,
}

impl<'a> Type<'a> {
  pub fn display<'m>(&'a self, module: &'m Module) -> TypeDisplay<'a, 'm> {
    TypeDisplay(self, module)
  }
}


pub struct TypeDisplay<'a, 'm>(pub &'a Type<'a>, pub &'m Module<'m,'m>);

impl<'a, 'm> fmt::Display for TypeDisplay<'a, 'm> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let ty = self.0;
    let mol = self.1;

    match &ty.vari {
      TypeVari::Nick(s) => {
        write!(f, "{}{}{}{:x}",
          "\"".yellow().bold(),
          mol.nick_map[s.idx as usize].yellow().bold(),
          "\"".yellow().bold(),
          s.idx
        )?;
      }

      TypeVari::Path(path) => {
        write!(f, "{} ", "path".blue().bold())?;

        for (i, x) in path.iter().enumerate() {
          write!(f, "{x}", )?;
          if i + 1 < path.len() {
            write!(f, "{}", "::".bright_black())?;
          }
        }
      }

      TypeVari::Void => write!(f, "{}", "void".blue().bold())?,
      TypeVari::Null => write!(f, "{}", "null_t".blue().bold())?,

      TypeVari::Bool => write!(f, "{}", "bool".blue().bold())?,
      TypeVari::Char => write!(f, "{}", "char".blue().bold())?,
      TypeVari::Ptr  => write!(f, "{}", "ptr".blue().bold())?,

      TypeVari::ArchSize{sig} => write!(f, "{}", format!("{}{}", if *sig {"i"} else {"u"}, "size").blue().bold())?,

      TypeVari::Int{bit, sig} => write!(f, "{}", format!("{}{}", if *sig {"i"} else {"u"}, *bit).blue().bold())?,
      TypeVari::Float{bit} => write!(f, "{}", format!("f{}", *bit).blue().bold())?,

      TypeVari::Function(s) => {
        write!(f, "{}{}", "fun".blue().bold(), "(".bright_black())?;
        
        for (i, x) in s.args.iter().enumerate() {
          write!(f, "{}{} {}", x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < s.args.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", ")".bright_black())?;
        
        if s.is_static {
          write!(f, "{}", " static".green().bold())?;
        }
        
        if s.is_const {
          write!(f, "{}", " const".green().bold())?;
        }

        write!(f, "{}", " -> ".bright_black())?;

        write!(f, "{}", mol.get_type(s.ret).display(mol))?;
      }

      TypeVari::Struct(s) => {
        write!(f, "{}{}", "struct".blue().bold(), "{".bright_black())?;

        for (i, x) in s.vars.iter().enumerate() {
          write!(f, "{} {}{} {}", x.vis, x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < s.vars.len() || !s.funs.is_empty() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        for (i, x) in s.funs.iter().enumerate() {
          let fun_decl = mol.get_decl(*x);
          write!(f, "{} {}{}", fun_decl.vis, "fun ".blue().bold(), fun_decl.name.to_string().blue().bold())?;
          
          if let crate::ast::DeclVari::Fun(fdecl) = &fun_decl.vari {
             write!(f, ": {}", mol.get_type(fdecl.kind).display(mol))?;
          }
          if i + 1 < s.funs.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", "}".bright_black())?;
      }

      TypeVari::Iface(s) => {
        write!(f, "{}{}", "iface".blue().bold(), "{".bright_black())?;

        for (i, x) in s.funs.iter().enumerate() {
          write!(f, "{} {}{} {}", x.vis, x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < s.funs.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", "}".bright_black())?;
      }

      TypeVari::Trait(s) => {
        write!(f, "{}{}", "trait".blue().bold(), "{".bright_black())?;

        for (i, x) in s.funs.iter().enumerate() {
          write!(f, "{} {}{} {}", x.vis, x.name.str().blue().bold(), ":".bright_black(), mol.get_type(x.kind).display(mol))?;
          if i + 1 < s.funs.len() {
            write!(f, "{} ", ";".bright_black())?;
          }
        }

        write!(f, "{}", "}".bright_black())?;
      }

      TypeVari::ReferenceOf{sub, acc} => {
        let acc_str = match acc {
          AccessKind::IMM => "imm ",
          AccessKind::MUT => "mut ",
        };
        let sub_ty = mol.get_type(*sub);
        match &sub_ty.vari {
          TypeVari::Struct(_) | TypeVari::Iface(_) => {
            write!(f, "{}{}{}", "&".blue().bold(), acc_str.blue().bold(), sub)?;
          }
          _ => {
            write!(f, "{}{}{}", "&".blue().bold(), acc_str.blue().bold(), sub_ty.display(mol))?;
          }
        }
      }

      TypeVari::PointerOf{sub, acc} => {
        let acc_str = match acc {
          AccessKind::IMM => "imm ",
          AccessKind::MUT => "mut ",
        };
        let sub_ty = mol.get_type(*sub);
        match &sub_ty.vari {
          TypeVari::Struct(_) | TypeVari::Iface(_) => {
            write!(f, "{}{}{}", "^".blue().bold(), acc_str.blue().bold(), sub)?;
          }
          _ => {
            write!(f, "{}{}{}", "^".blue().bold(), acc_str.blue().bold(), sub_ty.display(mol))?;
          }
        }
      }

      TypeVari::Enum(e) => {
        write!(f, "{}{}", "enum".blue().bold(), "{".bright_black())?;
        for (i, x) in e.vals.iter().enumerate() {
          write!(f, "{}", x.name.str().blue().bold())?;
          let val_str = match x.val {
            IntegerValue::SIG(v) => format!("{}", v),
            IntegerValue::USG(v) => format!("{}", v),
          };
          write!(f, " {} {}", "=".bright_black(), val_str.green())?;
          if i + 1 < e.vals.len() {
            write!(f, "{} ", ",".bright_black())?;
          }
        }
        write!(f, "{}", "}".bright_black())?;
      }

      TypeVari::Flags(e) => {
        write!(f, "{}{}", "flags".blue().bold(), "{".bright_black())?;
        for (i, x) in e.vals.iter().enumerate() {
          write!(f, "{}", x.name.str().blue().bold())?;
          let val_str = match x.val {
            IntegerValue::SIG(v) => format!("{}", v),
            IntegerValue::USG(v) => format!("{}", v),
          };
          write!(f, " {} {}", "=".bright_black(), val_str.green())?;
          if i + 1 < e.vals.len() {
            write!(f, "{} ", ",".bright_black())?;
          }
        }
        write!(f, "{}", "}".bright_black())?;
      }

      _ => write!(f, "unknown")?,
    }

    Ok(())
  }
}
