use crate::mir::{BlokId, TypeId};


pub struct Func {
  pub name: String,
  pub ret_ty: TypeId,
  pub arg_tys: Vec<TypeId>,
  pub blocks: Vec<BlokId>,
}
