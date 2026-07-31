use super::identy::HirId;

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


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HirFloatSize {
  F16, F32, F64, F128,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirTypeVari {
  Int{bit: u32, sig: bool},
  Float{bit: HirFloatSize},
  Bool,
  Char,
  Ptr,
  Void,
  Null,

  ZArrayOf(HirId),
  PArrayOf(HirPArrayType),

  Function(HirFunType),
  Struct(HirStructType),
  Iface(HirIfaceType),
  SelfType,
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
      HirTypeVari::Int{bit, sig} => write!(f, "{}", format!("{}{}", if *sig {"i"} else {"u"}, *bit).blue().bold())?,
      HirTypeVari::Float{bit} => {
        write!(f, "{}", match *bit {
          HirFloatSize::F16  => "f16",
          HirFloatSize::F32  => "f32",
          HirFloatSize::F64  => "f64",
          HirFloatSize::F128 => "f128",
        }.blue().bold())?;
      }

      HirTypeVari::Bool => write!(f, "{}", "bool".blue().bold())?,
      HirTypeVari::Char => write!(f, "{}", "char".blue().bold())?,
      HirTypeVari::Ptr => write!(f, "{}", "ptr".blue().bold())?,
      HirTypeVari::Void => write!(f, "{}", "void".blue().bold())?,
      HirTypeVari::Null => write!(f, "{}", "null".blue().bold())?,
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
      
      HirTypeVari::Iface(s) => {
        write!(f, "{}{}", "iface".blue().bold(), "{".bright_black())?;

        for (i, v) in s.funs.iter().enumerate() {
          write!(f, "{}{} {}", v.name.blue().bold(), ":".bright_black(), v.kind)?;
          if i + 1 < s.funs.len() { write!(f, "{} ", ",".bright_black())?; }
        }

        write!(f, "{}", "}".bright_black())?;
      }
      
      HirTypeVari::SelfType => write!(f, "{}", "Self".blue().bold().underline())?,
    };

    Ok(())
  }
}
