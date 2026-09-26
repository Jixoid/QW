use std::num::NonZeroU32;

use qwc_mir::{Krate, Layout, LayoutBy, LayoutInfo, Type, TypeId, TypeKind};


pub struct TypeInterner<'a> {
  pub layinfo: &'a LayoutInfo,
  
  ty_unit: TypeId,

  ty_bool: TypeId,

  // Hash
  ty_ptr: TypeId,
  
  // Int
  ty_i8: TypeId,
  ty_i16: TypeId,
  ty_i32: TypeId,
  ty_i64: TypeId,
  ty_i128: TypeId,
}


impl<'a> TypeInterner<'a> {
  pub fn new(cre: &mut Krate, layinfo: &'a LayoutInfo) -> Self {
    Self {
      layinfo,

      ty_unit: cre.push(Type{kind: TypeKind::Unit, layout: Layout::new_zst(LayoutBy::QW)}),

      ty_bool: cre.push(Type{kind: TypeKind::Bool, layout: Layout::new_sst(1, 1, LayoutBy::QW)}),

      ty_ptr: cre.push(Type{kind: TypeKind::Ptr, layout: layinfo.ptr_size}),
      
      // Int
      ty_i8:   cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(8)},   true), layout: layinfo.i8_lay}),
      ty_i16:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(16)},  true), layout: layinfo.i16_lay}),
      ty_i32:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(32)},  true), layout: layinfo.i32_lay}),
      ty_i64:  cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(64)},  true), layout: layinfo.i64_lay}),
      ty_i128: cre.push(Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(128)}, true), layout: layinfo.i128_lay}),
    }
  }

  
  pub fn ty_unit(&self) -> TypeId { self.ty_unit }

  pub fn ty_bool(&self) -> TypeId { self.ty_bool }

  pub fn ty_ptr(&self) -> TypeId { self.ty_ptr }


  // Int
  pub fn ty_i8(&self) -> TypeId   { self.ty_i8 }
  pub fn ty_i16(&self) -> TypeId  { self.ty_i16 }
  pub fn ty_i32(&self) -> TypeId  { self.ty_i32 }
  pub fn ty_i64(&self) -> TypeId  { self.ty_i64 }
  pub fn ty_i128(&self) -> TypeId { self.ty_i128 }
}
