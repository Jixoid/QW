/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::NonZeroU32;

use qwc_diagnostic::{Message, Status::{self, Success}, Summary, msg::THE_ADDRESS_SPACE_X_WAS_NOT_FOUND_IN_THIS_TARGET, ret_err};
use qwc_mir::{self as mir, FloatKind, Krate, Layout, LayoutInfo, Type, TypeId, TypeKind, id::PushOkApi};
use rustc_hash::FxHashMap;

use crate::layout::Layouter;


pub struct TypeInterner<'a> {
  pub layinfo: &'a LayoutInfo,
  
  ty_unit: TypeId,

  ty_bool: TypeId,

  // Ptr
  ty_ptr: FxHashMap<u16, TypeId>,
  ty_fatptr: FxHashMap<u16, TypeId>,
  ty_fatptrint: FxHashMap<u16, TypeId>,
  
  // Int
  ty_i8: TypeId,
  ty_i16: TypeId,
  ty_i32: TypeId,
  ty_i64: TypeId,
  ty_i128: TypeId,
  
  // Float
  ty_f16: TypeId,
  ty_f32: TypeId,
  ty_f64: TypeId,
  ty_f128: TypeId,
}


impl<'a> TypeInterner<'a> {
  pub fn new(cre: &mut Krate, layinfo: &'a LayoutInfo) -> Self {
    Self {
      layinfo,

      ty_unit: Type{kind: TypeKind::Unit, layout: Layout::new_zst()}.push(cre),

      ty_bool: Type{kind: TypeKind::Bool, layout: Layout::new_sst(1, 1)}.push(cre),

      ty_ptr: FxHashMap::default(),
      ty_fatptr: FxHashMap::default(),
      ty_fatptrint: FxHashMap::default(),
      
      // Int
      ty_i8:   Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(8)},   true), layout: *layinfo.int.get(&8).unwrap()}.push(cre),
      ty_i16:  Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(16)},  true), layout: *layinfo.int.get(&16).unwrap()}.push(cre),
      ty_i32:  Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(32)},  true), layout: *layinfo.int.get(&32).unwrap()}.push(cre),
      ty_i64:  Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(64)},  true), layout: *layinfo.int.get(&64).unwrap()}.push(cre),
      ty_i128: Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(128)}, true), layout: *layinfo.int.get(&128).unwrap()}.push(cre),

      // Float
      ty_f16:  Type{kind: TypeKind::Float(mir::FloatKind::F16),  layout: *layinfo.float.get(&FloatKind::F16).unwrap()}.push(cre),
      ty_f32:  Type{kind: TypeKind::Float(mir::FloatKind::F32),  layout: *layinfo.float.get(&FloatKind::F32).unwrap()}.push(cre),
      ty_f64:  Type{kind: TypeKind::Float(mir::FloatKind::F64),  layout: *layinfo.float.get(&FloatKind::F64).unwrap()}.push(cre),
      ty_f128: Type{kind: TypeKind::Float(mir::FloatKind::F128), layout: *layinfo.float.get(&FloatKind::F128).unwrap()}.push(cre),
    }
  }


  // Primitive
  pub fn ty_unit(&self) -> TypeId { self.ty_unit }
  pub fn ty_bool(&self) -> TypeId { self.ty_bool }


  // Ptr(s)
  pub fn ty_ptr(&mut self, cre: &mut Krate, sum: &mut Summary, addrspace: u16) -> Status<TypeId> {
    use std::collections::hash_map::Entry::*;
    match self.ty_ptr.entry(addrspace) {
      Occupied(entry) => Success(*entry.get()),

      Vacant(entry) => {
        let layout = self.layinfo.ptr.get(&addrspace);

        let Some(&layout) = layout else {
          ret_err!(sum, Message::error_nopos(THE_ADDRESS_SPACE_X_WAS_NOT_FOUND_IN_THIS_TARGET.args(&[&addrspace.to_string()])))
        };

        let id = Type{kind: TypeKind::Ptr(addrspace), layout}.push(cre);

        entry.insert(id);
        Success(id)
      }
    }
  }

  pub fn ty_fatptr(&mut self, cre: &mut Krate, sum: &mut Summary, addrspace: u16) -> Status<TypeId> {
    let ty_ptr = self.ty_ptr(cre, sum, addrspace)?;

    use std::collections::hash_map::Entry::*;
    match self.ty_fatptr.entry(addrspace) {
      Occupied(entry) => Success(*entry.get()),

      Vacant(entry) => {
        let id = {
          let kind = TypeKind::Struct(cre.extra(&[ty_ptr, ty_ptr]));
          let layout = Layouter::layout(sum, &kind, self.layinfo, cre)?;

          Type{kind, layout}.push(cre)
        };

        entry.insert(id);
        Success(id)
      }
    }
  }

  pub fn ty_fatptrint(&mut self, cre: &mut Krate, sum: &mut Summary, addrspace: u16) -> Status<TypeId> {
    let ty_ptr = self.ty_ptr(cre, sum, addrspace)?;
    let ty_int = self.ty_arch_int();

    use std::collections::hash_map::Entry::*;
    match self.ty_fatptrint.entry(addrspace) {
      Occupied(entry) => Success(*entry.get()),

      Vacant(entry) => {
        let id = {
          let kind = TypeKind::Struct(cre.extra(&[ty_ptr, ty_int]));
          let layout = Layouter::layout(sum, &kind, self.layinfo, cre)?;

          Type{kind, layout}.push(cre)
        };

        entry.insert(id);
        Success(id)
      }
    }
  }


  // Int
  pub fn ty_arch_int(&self) -> TypeId {
    match self.layinfo.nint.unwrap() {
      8  => self.ty_i8,
      16 => self.ty_i16,
      32 => self.ty_i32,
      64 => self.ty_i64,
      _ => panic!()
    }
  }

  pub fn ty_i8(&self) -> TypeId   { self.ty_i8 }
  pub fn ty_i16(&self) -> TypeId  { self.ty_i16 }
  pub fn ty_i32(&self) -> TypeId  { self.ty_i32 }
  pub fn ty_i64(&self) -> TypeId  { self.ty_i64 }
  pub fn ty_i128(&self) -> TypeId { self.ty_i128 }

  // Float
  pub fn ty_f16(&self) -> TypeId  { self.ty_f16 }
  pub fn ty_f32(&self) -> TypeId  { self.ty_f32 }
  pub fn ty_f64(&self) -> TypeId  { self.ty_f64 }
  pub fn ty_f128(&self) -> TypeId { self.ty_f128 }
}
