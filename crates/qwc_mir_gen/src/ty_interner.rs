use std::num::NonZeroU32;

use qwc_mir::{Krate, Layout, LayoutBy, LayoutInfo, Type, TypeId, TypeKind};
use rustc_hash::FxHashMap;


pub struct TypeInterner<'a> {
  pub layinfo: &'a LayoutInfo,
  
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


impl<'a> TypeInterner<'a> {
  pub fn new(cre: &mut Krate, layinfo: &'a LayoutInfo) -> Self {
    Self {
      layinfo,

      ty_unit: cre.push(Type{kind: TypeKind::Unit, layout: Layout::new(0, 1, LayoutBy::QW)}),

      ty_bool: cre.push(Type{kind: TypeKind::Bool, layout: Layout::new(1, 1, LayoutBy::QW)}),

      // Hash
      ty_ptr: FxHashMap::default(),
      
      // Int
      ty_i8:   cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(8)},   true), layout: layinfo.i8_lay}),
      ty_i16:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(16)},  true), layout: layinfo.i16_lay}),
      ty_i32:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(32)},  true), layout: layinfo.i32_lay}),
      ty_i64:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(64)},  true), layout: layinfo.i64_lay}),
      ty_i128: cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(128)}, true), layout: layinfo.i128_lay}),

      ty_u8:   cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(8)},   false), layout: layinfo.i8_lay}),
      ty_u16:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(16)},  false), layout: layinfo.i16_lay}),
      ty_u32:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(32)},  false), layout: layinfo.i32_lay}),
      ty_u64:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(64)},  false), layout: layinfo.i64_lay}),
      ty_u128: cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(128)}, false), layout: layinfo.i128_lay}),
    }
  }

  
  pub fn ty_unit(&self) -> TypeId { self.ty_unit }

  pub fn ty_bool(&self) -> TypeId { self.ty_bool }

  
  // Hash
  pub fn ty_ptr(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_ptr.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Ptr(id),
          layout: self.layinfo.ptr_size,
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }


  // Int
  pub fn ty_i8(&self) -> TypeId   { self.ty_i8 }
  pub fn ty_i16(&self) -> TypeId  { self.ty_i16 }
  pub fn ty_i32(&self) -> TypeId  { self.ty_i32 }
  pub fn ty_i64(&self) -> TypeId  { self.ty_i64 }
  pub fn ty_i128(&self) -> TypeId { self.ty_i128 }

  pub fn ty_u8(&self) -> TypeId   { self.ty_u8 }
  pub fn ty_u16(&self) -> TypeId  { self.ty_u16 }
  pub fn ty_u32(&self) -> TypeId  { self.ty_u32 }
  pub fn ty_u64(&self) -> TypeId  { self.ty_u64 }
  pub fn ty_u128(&self) -> TypeId { self.ty_u128 }
}
