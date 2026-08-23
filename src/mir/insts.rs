use crate::mir::{BlokId, Ptr, TypeId, ValuId};


pub enum ICmp {
  /// Equal
  EQ,
  /// Not Equal
  NE,
  
  /// Greater Than
  GT,
  /// Greater or Equal
  GE,
  /// Less Than
  LT,
  /// Less or Equal
  LE,
}

pub enum FCmp {
  /// Always False
  FALSE,
  /// Always True
  TRUE,

  /// Equal
  EQ,
  /// Not Equal
  NE,
  /// Greater Than
  GT,
  /// Greater or Equal
  GE,
  /// Less Than
  LT,
  /// Less or Equal
  LE,
  /// Check
  RD,
}


pub enum InstVari {
  // Stack
  Alloca(TypeId),
  Load(TypeId, Ptr),
  Store(ValuId, Ptr),

  // GEP
  GetElementPtr(Ptr, Vec<u32>),

  // Arithmetic
  IAdd(ValuId, ValuId),
  ISub(ValuId, ValuId),
  IMul(ValuId, ValuId),
  IDiv(ValuId, ValuId),
  IRem(ValuId, ValuId),

  FAdd(ValuId, ValuId),
  FSub(ValuId, ValuId),
  FMul(ValuId, ValuId),
  FDiv(ValuId, ValuId),

  // Compare
  ICmp(ValuId, ValuId, ICmp, bool /* signed */),
  FCmp(ValuId, ValuId, FCmp, bool /* ordered */),

  // Bitwise
  And(ValuId, ValuId),
  Or (ValuId, ValuId),
  Xor(ValuId, ValuId),
  Shl(ValuId, ValuId),
  Shr(ValuId, ValuId),

  // Call
  Call(ValuId, Vec<ValuId>),

  // Cast
  Bitcast(ValuId, TypeId),
  Trunc(ValuId, TypeId),
  ZExt(ValuId, TypeId),
  SExt(ValuId, TypeId),
  IntToPtr(ValuId, TypeId),
  PtrToInt(ValuId, TypeId),

  // Branch
  Br(BlokId),
  CondBr(ValuId, BlokId, BlokId),
  Ret(Option<ValuId>),
}

pub struct Inst {
  pub vari: InstVari,
  pub ty: Option<TypeId>,
}
