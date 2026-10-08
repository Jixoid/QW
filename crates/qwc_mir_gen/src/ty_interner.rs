/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::NonZeroU32;

use qwc_mir::{self as mir, Krate, Layout, LayoutBy, LayoutInfo, Type, TypeId, TypeKind, id::PushOkApi};

use crate::layout::Layouter;


pub struct TypeInterner<'a> {
  pub layinfo: &'a LayoutInfo,
  
  ty_unit: TypeId,

  ty_bool: TypeId,

  // Ptr
  ty_ptr: TypeId,
  ty_fatptr: TypeId,
  ty_fatptrint: TypeId,
  
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
    let ty_i8  = Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(8)},  true), layout: layinfo.i8_lay}.push(cre);
    let ty_i16 = Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(16)}, true), layout: layinfo.i16_lay}.push(cre);
    let ty_i32 = Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(32)}, true), layout: layinfo.i32_lay}.push(cre);
    let ty_i64 = Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(64)}, true), layout: layinfo.i64_lay}.push(cre);

    let ty_arch_int = || -> TypeId {
      use mir::ArchBit::*;

      match layinfo.arch_bit {
        B8  => ty_i8,
        B16 => ty_i16,
        B32 => ty_i32,
        B64 => ty_i64,
      }
    };


    let ty_ptr = Type{kind: TypeKind::Ptr, layout: layinfo.ptr_size}.push(cre);

    let ty_fatptr = {
      let kind = TypeKind::Struct(cre.extra(&[ty_ptr, ty_ptr]));
      let layout = Layouter::layout(&kind, layinfo, cre, qwc_hir::LayoutBy::SYS);

      Type{kind, layout}.push(cre)
    };

    let ty_fatptrint = {
      let kind = TypeKind::Struct(cre.extra(&[ty_ptr, ty_arch_int()]));
      let layout = Layouter::layout(&kind, layinfo, cre, qwc_hir::LayoutBy::SYS);

      Type{kind, layout}.push(cre)
    };


    Self {
      layinfo,

      ty_unit: Type{kind: TypeKind::Unit, layout: Layout::new_zst(LayoutBy::QW)}.push(cre),

      ty_bool: Type{kind: TypeKind::Bool, layout: Layout::new_sst(1, 1, LayoutBy::QW)}.push(cre),

      ty_ptr,
      ty_fatptr,
      ty_fatptrint,
      
      // Int
      ty_i8:   Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(8)},   true), layout: layinfo.i8_lay}.push(cre),
      ty_i16:  Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(16)},  true), layout: layinfo.i16_lay}.push(cre),
      ty_i32:  Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(32)},  true), layout: layinfo.i32_lay}.push(cre),
      ty_i64:  Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(64)},  true), layout: layinfo.i64_lay}.push(cre),
      ty_i128: Type{kind: TypeKind::Int(unsafe {NonZeroU32::new_unchecked(128)}, true), layout: layinfo.i128_lay}.push(cre),

      // Float
      ty_f16:  Type{kind: TypeKind::Float(mir::FloatKind::F16), layout: layinfo.f16_lay}.push(cre),
      ty_f32:  Type{kind: TypeKind::Float(mir::FloatKind::F32), layout: layinfo.f32_lay}.push(cre),
      ty_f64:  Type{kind: TypeKind::Float(mir::FloatKind::F64), layout: layinfo.f64_lay}.push(cre),
      ty_f128: Type{kind: TypeKind::Float(mir::FloatKind::F128), layout: layinfo.f128_lay}.push(cre),
    }
  }

  
  pub fn ty_unit(&self) -> TypeId { self.ty_unit }

  pub fn ty_bool(&self) -> TypeId { self.ty_bool }

  pub fn ty_ptr(&self) -> TypeId { self.ty_ptr }
  pub fn ty_fatptr(&self) -> TypeId { self.ty_fatptr }
  pub fn ty_fatptrint(&self) -> TypeId { self.ty_fatptrint }


  // Int
  pub fn ty_arch_int(&self) -> TypeId {
    use mir::ArchBit::*;

    match self.layinfo.arch_bit {
      B8  => self.ty_i8,
      B16 => self.ty_i16,
      B32 => self.ty_i32,
      B64 => self.ty_i64,
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
