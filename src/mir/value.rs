
pub type Ptr = usize;

pub enum Value {
  ConstInt(i64),
  ConstFloat(f64),
  ConstBool(bool),
  Null,
}
