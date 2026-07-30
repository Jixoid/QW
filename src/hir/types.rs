use super::identy::HirId;
use crate::ast::types::AccessKind;

use owo_colors::OwoColorize;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFieldType {
  pub name: String,
  pub kind: HirId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFieldCons {
  pub val: i128,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPArrayType {
  pub sub: HirId,
  pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFunType {
  pub args: Vec<HirFieldType>,
  pub ret: HirId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirStructType {
  pub base: Vec<HirId>,
  pub vars: Vec<HirFieldType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirEnumType {
  pub vals: Vec<HirFieldCons>,
  pub iset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirTypeVari {
  ISize, USize,
  I8, I16, I32, I64, I128,
  U8, U16, U32, U64, U128,
  F16, F32, F64, F128,
  Bool,
  Char,
  Ptr,
  Void,
  Null,

  PointerOf{ sub: HirId, acc: AccessKind },
  ReferenceOf{ sub: HirId, acc: AccessKind },
  ZArrayOf(HirId),
  PArrayOf(HirPArrayType),

  Function(HirFunType),
  Struct(HirStructType),
  Iface(HirIfaceType),
  Enum(HirEnumType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirIfaceType {
  pub funs: Vec<HirFieldType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirType {
  pub vari: HirTypeVari,
}

use core::fmt;

impl fmt::Display for HirTypeVari {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      HirTypeVari::ISize => write!(f, "{}", "isize".blue().bold())?,
      HirTypeVari::USize => write!(f, "{}", "usize".blue().bold())?,
      HirTypeVari::I8 => write!(f, "{}", "i8".blue().bold())?,
      HirTypeVari::I16 => write!(f, "{}", "i16".blue().bold())?,
      HirTypeVari::I32 => write!(f, "{}", "i32".blue().bold())?,
      HirTypeVari::I64 => write!(f, "{}", "i64".blue().bold())?,
      HirTypeVari::I128 => write!(f, "{}", "i128".blue().bold())?,
      HirTypeVari::U8 => write!(f, "{}", "u8".blue().bold())?,
      HirTypeVari::U16 => write!(f, "{}", "u16".blue().bold())?,
      HirTypeVari::U32 => write!(f, "{}", "u32".blue().bold())?,
      HirTypeVari::U64 => write!(f, "{}", "u64".blue().bold())?,
      HirTypeVari::U128 => write!(f, "{}", "u128".blue().bold())?,
      HirTypeVari::F16 => write!(f, "{}", "f16".blue().bold())?,
      HirTypeVari::F32 => write!(f, "{}", "f32".blue().bold())?,
      HirTypeVari::F64 => write!(f, "{}", "f64".blue().bold())?,
      HirTypeVari::F128 => write!(f, "{}", "f128".blue().bold())?,
      HirTypeVari::Bool => write!(f, "{}", "bool".blue().bold())?,
      HirTypeVari::Char => write!(f, "{}", "char".blue().bold())?,
      HirTypeVari::Ptr => write!(f, "{}", "ptr".blue().bold())?,
      HirTypeVari::Void => write!(f, "{}", "void".blue().bold())?,
      HirTypeVari::Null => write!(f, "{}", "null".blue().bold())?,
      HirTypeVari::PointerOf{sub, acc} => {
        let acc_str = match acc { AccessKind::IMM => "imm", AccessKind::MUT => "mut" };
        write!(f, "^{} {}", acc_str.green().bold(), sub)?;
      }
      HirTypeVari::ReferenceOf{sub, acc} => {
        let acc_str = match acc { AccessKind::IMM => "imm", AccessKind::MUT => "mut" };
        write!(f, "&{} {}", acc_str.green().bold(), sub)?;
      }
      HirTypeVari::ZArrayOf(sub) => write!(f, "[{}]", sub)?,
      HirTypeVari::PArrayOf(p) => write!(f, "[{},{}]", p.size, p.sub)?,
      HirTypeVari::Function(fun) => {
        write!(f, "{}{}", "fun".blue().bold(), "(".bright_black())?;
        for (i, arg) in fun.args.iter().enumerate() {
          write!(f, "{}{} {}", arg.name.blue().bold(), ":".bright_black(), arg.kind)?;
          if i + 1 < fun.args.len() { write!(f, ", ")?; }
        }
        write!(f, "{} {}", ") ->".bright_black(), fun.ret)?;
      }
      HirTypeVari::Struct(s) => {
        write!(f, "{}{}", "struct".blue().bold(), "{".bright_black())?;

        for (i, v) in s.vars.iter().enumerate() {
          write!(f, "{}{} {}", v.name.blue().bold(), ":".bright_black(), v.kind)?;
          if i + 1 < s.vars.len() { write!(f, "{} ", ",".bright_black())?; }
        }

        write!(f, "{}", "}".bright_black())?;
      }
      HirTypeVari::Enum(_) => {
        write!(f, "enum {{ .. }}")?;
      }
      HirTypeVari::Iface(s) => {
        write!(f, "{}{}", "iface".blue().bold(), "{".bright_black())?;

        for (i, v) in s.funs.iter().enumerate() {
          write!(f, "{}{} {}", v.name.blue().bold(), ":".bright_black(), v.kind)?;
          if i + 1 < s.funs.len() { write!(f, "{} ", ",".bright_black())?; }
        }

        write!(f, "{}", "}".bright_black())?;
      }
    };

    Ok(())
  }
}
