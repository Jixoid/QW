use core::fmt;
use owo_colors::OwoColorize;

use super::identy::HirId;
use super::value::HirValue;

#[derive(Debug, Clone, PartialEq)]
pub enum HirInstrVari {

  Alloca(HirId), // Ty_Id
  Load(HirId, HirValue), // Ty_Id, Ptr
  Store(HirValue, HirValue), // Val, Ptr
  GetElementPtr(HirId, HirValue, Vec<u32>), // ResultTy, Ptr, Indices


  Add(HirValue, HirValue),
  Sub(HirValue, HirValue),
  Mul(HirValue, HirValue),
  Div(HirValue, HirValue),


  ICmp(String, HirValue, HirValue), // Eq, Ne, Slt, Ugt, vs.
  FCmp(String, HirValue, HirValue), // Oeq, One, vs.


  And(HirValue, HirValue),
  Or(HirValue, HirValue),
  Xor(HirValue, HirValue),
  Shl(HirValue, HirValue),
  Shr(HirValue, HirValue),


  Call(HirValue, Vec<HirValue>),


  Bitcast(HirValue, HirId),
  Trunc(HirValue, HirId),
  ZExt(HirValue, HirId),
  SExt(HirValue, HirId),
  IntToPtr(HirValue, HirId),
  PtrToInt(HirValue, HirId),


  Br(HirId), // Block_Id
  CondBr(HirValue, HirId, HirId), // Cond, TrueBlock, FalseBlock
  Ret(Option<HirValue>), // Val
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirInstr {
  pub vari: HirInstrVari,
  pub ty: HirId,
}


impl fmt::Display for HirInstrVari {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      HirInstrVari::Alloca(ty_id) => write!(f, "{} {}", "alloca".blue().bold(), ty_id),
      HirInstrVari::Load(ty_id, ptr) => write!(f, "{} {} {} {}", "load".blue().bold(), ty_id, ",".bright_black(), ptr),
      HirInstrVari::Store(val, ptr) => write!(f, "{} {} {} {}", "store".blue().bold(), val, ",".bright_black(), ptr),
      HirInstrVari::Ret(opt_val) => {
        write!(f, "{}", "ret".blue().bold())?;
        if let Some(val) = opt_val {
          write!(f, " {}", val)?;
        }
        Ok(())
      },
      _ => write!(f, "{}", format!("{:?}", self).red()),
    }
  }
}

impl fmt::Display for HirInstr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {} {}", self.vari, "->".bright_black(), self.ty)
  }
}
