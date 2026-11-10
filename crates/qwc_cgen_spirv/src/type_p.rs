/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir::{self as mir, FloatKind, TypeId, TypeKind};
use rspirv::dr::Builder;
use rspirv::spirv::{StorageClass, Word};
use rustc_hash::FxHashMap;


pub struct TypeLow {
  type_cache: FxHashMap<TypeId, Word>,
  ptr_cache: FxHashMap<(TypeId, StorageClass), Word>,
}

impl TypeLow {
  pub fn new() -> Self {
    Self {
      type_cache: FxHashMap::default(),
      ptr_cache: FxHashMap::default(),
    }
  }

  pub fn low(&mut self, b: &mut Builder, cre: &mir::Krate, ty: TypeId) -> Result<Word, String> {
    if let Some(&w) = self.type_cache.get(&ty) {
      return Ok(w);
    }

    let it = cre.get(ty);
    let word = match it.kind {
      TypeKind::Unit => b.type_void(),
      TypeKind::Bool => b.type_bool(),
      TypeKind::Int(bits, signed) => {
        b.type_int(bits.get(), if signed { 1 } else { 0 })
      }
      TypeKind::Float(fk) => match fk {
        FloatKind::F16 | FloatKind::BF16 => b.type_float(16, None),
        FloatKind::F32 => b.type_float(32, None),
        FloatKind::F64 => b.type_float(64, None),
        FloatKind::F128 => return Err("128-bit float is not supported in SPIR-V".to_string()),
      },
      TypeKind::Ptr(space) => {
        // Default storage class based on address space (0: Function, 1: Uniform, 2: StorageBuffer, etc.)
        let sc = match space {
          1 => StorageClass::Uniform,
          2 => StorageClass::StorageBuffer,
          3 => StorageClass::Workgroup,
          _ => StorageClass::Function,
        };
        // For void / untyped pointers or generic pointers in SPIR-V, pointer to u8/i32
        let elem_ty = b.type_int(8, 0);
        b.type_pointer(None, sc, elem_ty)
      }
      TypeKind::Fun { args, ret } => {
        let ret_w = self.low(b, cre, ret)?;
        let mut arg_ws = Vec::new();
        for arg_id in cre.extra_get(args) {
          arg_ws.push(self.low(b, cre, arg_id)?);
        }
        b.type_function(ret_w, arg_ws)
      }
      TypeKind::Struct(fields) => {
        let mut field_ws = Vec::new();
        for field_id in cre.extra_get(fields) {
          field_ws.push(self.low(b, cre, field_id)?);
        }
        b.type_struct(field_ws)
      }
      TypeKind::Array(elem, len) => {
        let elem_w = self.low(b, cre, elem)?;
        let u32_ty = b.type_int(32, 0);
        let len_id = b.constant_bit32(u32_ty, len as u32);
        b.type_array(elem_w, len_id)
      }
      TypeKind::Slice(elem) => {
        // Slice is lowered as struct { ptr, len }
        let elem_w = self.low(b, cre, elem)?;
        let ptr_ty = b.type_pointer(None, StorageClass::Function, elem_w);
        let len_ty = b.type_int(64, 0);
        b.type_struct(vec![ptr_ty, len_ty])
      }
    };

    self.type_cache.insert(ty, word);
    Ok(word)
  }

  pub fn pointer_type(&mut self, b: &mut Builder, cre: &mir::Krate, elem_ty: TypeId, sc: StorageClass) -> Result<Word, String> {
    if let Some(&w) = self.ptr_cache.get(&(elem_ty, sc)) {
      return Ok(w);
    }
    let inner_w = self.low(b, cre, elem_ty)?;
    let ptr_w = b.type_pointer(None, sc, inner_w);
    self.ptr_cache.insert((elem_ty, sc), ptr_w);
    Ok(ptr_w)
  }
}
