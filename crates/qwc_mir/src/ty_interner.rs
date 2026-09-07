use rustc_hash::FxHashMap;

use crate::{Krate, Type, TypeId};


pub struct TypeInterner {
  ty_unit: TypeId,

  ty_bool: TypeId,

  // Hash
  ty_ptr: FxHashMap<TypeId, TypeId>,
  
  // Int
  ty_i8: TypeId,
  ty_i16: TypeId,
  ty_i32: TypeId,
  ty_i64: TypeId,
  ty_i128: TypeId,

  ty_u8: TypeId,
  ty_u16: TypeId,
  ty_u32: TypeId,
  ty_u64: TypeId,
  ty_u128: TypeId,
}


impl TypeInterner {
  pub fn new(cre: &mut Krate) -> Self {
    Self {
      ty_unit: cre.push(Type::Unit),

      ty_bool: cre.push(Type::Bool),

      // Hash
      ty_ptr: FxHashMap::default(),
      
      // Int
      ty_i8: cre.push(Type::Int(8, true)),
      ty_i16: cre.push(Type::Int(16, true)),
      ty_i32: cre.push(Type::Int(32, true)),
      ty_i64: cre.push(Type::Int(64, true)),
      ty_i128: cre.push(Type::Int(128, true)),

      ty_u8: cre.push(Type::Int(8, false)),
      ty_u16: cre.push(Type::Int(16, false)),
      ty_u32: cre.push(Type::Int(32, false)),
      ty_u64: cre.push(Type::Int(64, false)),
      ty_u128: cre.push(Type::Int(128, false)),
    }
  }
}


impl Krate {
  pub fn ty_unit(&self) -> TypeId { self.tyin.as_ref().unwrap().ty_unit }

  pub fn ty_bool(&self) -> TypeId { self.tyin.as_ref().unwrap().ty_bool }

  // Hash
  pub fn ty_ptr(&mut self, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.tyin.as_mut().unwrap().ty_ptr.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type::Ptr(id);

        let id = TypeId::new(u32::try_from(self.list_type.push(this)).unwrap());

        entry.insert(id);
        id
      }
    }
  }


  // Int
  pub fn ty_i8(&self) -> TypeId   { self.tyin.as_ref().unwrap().ty_i8 }
  pub fn ty_i16(&self) -> TypeId  { self.tyin.as_ref().unwrap().ty_i16 }
  pub fn ty_i32(&self) -> TypeId  { self.tyin.as_ref().unwrap().ty_i32 }
  pub fn ty_i64(&self) -> TypeId  { self.tyin.as_ref().unwrap().ty_i64 }
  pub fn ty_i128(&self) -> TypeId { self.tyin.as_ref().unwrap().ty_i128 }

  pub fn ty_u8(&self) -> TypeId   { self.tyin.as_ref().unwrap().ty_u8 }
  pub fn ty_u16(&self) -> TypeId  { self.tyin.as_ref().unwrap().ty_u16 }
  pub fn ty_u32(&self) -> TypeId  { self.tyin.as_ref().unwrap().ty_u32 }
  pub fn ty_u64(&self) -> TypeId  { self.tyin.as_ref().unwrap().ty_u64 }
  pub fn ty_u128(&self) -> TypeId { self.tyin.as_ref().unwrap().ty_u128 }
}
