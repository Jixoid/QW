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
      HirInstrVari::GetElementPtr(ty_id, ptr, indices) => {
        write!(f, "{} {} {} {}", "getelementptr".blue().bold(), ty_id, ",".bright_black(), ptr)?;
        for index in indices {
          write!(f, " {} {}", ",".bright_black(), index)?;
        }
        Ok(())
      }
      HirInstrVari::Add(v1, v2) => write!(f, "{} {} {} {}", "add".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Sub(v1, v2) => write!(f, "{} {} {} {}", "sub".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Mul(v1, v2) => write!(f, "{} {} {} {}", "mul".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Div(v1, v2) => write!(f, "{} {} {} {}", "div".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::ICmp(cond, v1, v2) => write!(f, "{} {} {} {} {}", "icmp".blue().bold(), cond.as_str().cyan(), v1, ",".bright_black(), v2),
      HirInstrVari::FCmp(cond, v1, v2) => write!(f, "{} {} {} {} {}", "fcmp".blue().bold(), cond.as_str().cyan(), v1, ",".bright_black(), v2),
      HirInstrVari::And(v1, v2) => write!(f, "{} {} {} {}", "and".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Or(v1, v2) => write!(f, "{} {} {} {}", "or".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Xor(v1, v2) => write!(f, "{} {} {} {}", "xor".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Shl(v1, v2) => write!(f, "{} {} {} {}", "shl".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Shr(v1, v2) => write!(f, "{} {} {} {}", "shr".blue().bold(), v1, ",".bright_black(), v2),
      HirInstrVari::Call(func, args) => {
        write!(f, "{} {} {}", "call".blue().bold(), func, "(".bright_black())?;
        for (i, arg) in args.iter().enumerate() {
          if i > 0 { write!(f, "{}", ", ".bright_black())?; }
          write!(f, "{}", arg)?;
        }
        write!(f, "{}", ")".bright_black())
      }
      HirInstrVari::Bitcast(val, ty) => write!(f, "{} {} {} {}", "bitcast".blue().bold(), val, "to".bright_black(), ty),
      HirInstrVari::Trunc(val, ty) => write!(f, "{} {} {} {}", "trunc".blue().bold(), val, "to".bright_black(), ty),
      HirInstrVari::ZExt(val, ty) => write!(f, "{} {} {} {}", "zext".blue().bold(), val, "to".bright_black(), ty),
      HirInstrVari::SExt(val, ty) => write!(f, "{} {} {} {}", "sext".blue().bold(), val, "to".bright_black(), ty),
      HirInstrVari::IntToPtr(val, ty) => write!(f, "{} {} {} {}", "inttoptr".blue().bold(), val, "to".bright_black(), ty),
      HirInstrVari::PtrToInt(val, ty) => write!(f, "{} {} {} {}", "ptrtoint".blue().bold(), val, "to".bright_black(), ty),
      HirInstrVari::Br(block) => write!(f, "{} {}", "br".blue().bold(), block),
      HirInstrVari::CondBr(cond, true_block, false_block) => write!(f, "{} {} {} {} {} {}", "br".blue().bold(), cond, ",".bright_black(), true_block, ",".bright_black(), false_block),
      HirInstrVari::Ret(opt_val) => {
        write!(f, "{}", "ret".blue().bold())?;
        if let Some(val) = opt_val {
          write!(f, " {}", val)?;
        }
        Ok(())
      }
    }
  }
}

impl fmt::Display for HirInstr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {} {}", self.vari, "->".bright_black(), self.ty)
  }
}
