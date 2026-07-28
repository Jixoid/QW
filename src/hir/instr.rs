use super::identy::HirId;
use super::value::HirValue;

#[derive(Debug, Clone, PartialEq)]
pub enum HirInstrVari {
  // Memory
  Alloca(HirId), // Ty_Id
  Load(HirId, HirValue), // Ty_Id, Ptr
  Store(HirValue, HirValue), // Val, Ptr
  GetElementPtr(HirId, HirValue, Vec<u32>), // ResultTy, Ptr, Indices

  // Arithmetic
  Add(HirValue, HirValue),
  Sub(HirValue, HirValue),
  Mul(HirValue, HirValue),
  Div(HirValue, HirValue),

  // Comparison
  ICmp(String, HirValue, HirValue), // Eq, Ne, Slt, Ugt, vs.
  FCmp(String, HirValue, HirValue), // Oeq, One, vs.

  // Logic/Bitwise
  And(HirValue, HirValue),
  Or(HirValue, HirValue),
  Xor(HirValue, HirValue),
  Shl(HirValue, HirValue),
  Shr(HirValue, HirValue),

  // Calls
  Call(HirValue, Vec<HirValue>),

  // Type Casts
  Bitcast(HirValue, HirId),
  Trunc(HirValue, HirId),
  ZExt(HirValue, HirId),
  SExt(HirValue, HirId),
  IntToPtr(HirValue, HirId),
  PtrToInt(HirValue, HirId),

  // Terminators
  Br(HirId), // Block_Id
  CondBr(HirValue, HirId, HirId), // Cond, TrueBlock, FalseBlock
  Ret(Option<HirValue>), // Val
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirInstr {
  pub vari: HirInstrVari,
  pub ty: HirId,
}
