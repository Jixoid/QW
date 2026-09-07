
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Value {
  // NST
  Unit,
  
  // Primitive
  Bool(bool),

  I32(i32),
  I64(i64),

  // Raw
  //Raw(ThinVec<u8>)
}
