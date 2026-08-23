use crate::mir::{TypeId, Value};


pub struct Glob {
  pub name: String,
  pub ty: TypeId,
  pub init: Option<Value>,
}
