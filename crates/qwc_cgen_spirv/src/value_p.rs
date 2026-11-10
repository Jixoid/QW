/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir::{self as mir, Const, Value};
use rspirv::dr::Builder;
use rspirv::spirv::Word;
use rustc_apfloat::Float;
use rustc_hash::FxHashMap;

use crate::type_p::TypeLow;


pub struct FunCtx {
  pub ssa_map: FxHashMap<mir::SSA, Word>,
  pub param_map: Vec<Word>,
}

impl FunCtx {
  pub fn new() -> Self {
    Self {
      ssa_map: FxHashMap::default(),
      param_map: Vec::new(),
    }
  }
}

pub struct ValueLow;

impl ValueLow {
  pub fn low(
    b: &mut Builder,
    cre: &mir::Krate,
    ty_low: &mut TypeLow,
    symbols: &FxHashMap<mir::SymbId, Word>,
    fctx: &FunCtx,
    val: &Value,
    hint_ty: Option<mir::TypeId>,
  ) -> Result<Word, String> {
    match val {
      Value::SSA(ssa) => {
        fctx.ssa_map.get(ssa).copied().ok_or_else(|| {
          format!("SPIR-V: undefined SSA variable %{}", ssa.get())
        })
      }
      Value::Param(idx) => {
        fctx.param_map.get(*idx as usize).copied().ok_or_else(|| {
          format!("SPIR-V: undefined parameter index {}", idx)
        })
      }
      Value::GlobalRef(sym_id) => {
        symbols.get(sym_id).copied().ok_or_else(|| {
          format!("SPIR-V: undefined global reference for symbol ID {:?}", sym_id)
        })
      }
      Value::Const(c) => match c {
        Const::Unit => {
          let u32_ty = b.type_int(32, 0);
          Ok(b.constant_bit32(u32_ty, 0))
        }
        Const::Bool(v) => {
          let bool_ty = b.type_bool();
          Ok(if *v { b.constant_true(bool_ty) } else { b.constant_false(bool_ty) })
        }
        Const::I8(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_int(8, 1));
          Ok(b.constant_bit32(ty, *v as u32))
        }
        Const::I16(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_int(16, 1));
          Ok(b.constant_bit32(ty, *v as u32))
        }
        Const::I32(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_int(32, 1));
          Ok(b.constant_bit32(ty, *v))
        }
        Const::I64(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_int(64, 1));
          Ok(b.constant_bit64(ty, *v))
        }
        Const::I128(_v) => {
          Err("SPIR-V does not natively support 128-bit integer constants".to_string())
        }
        Const::F16(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_float(16, None));
          Ok(b.constant_bit32(ty, v.to_bits() as u32))
        }
        Const::F32(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_float(32, None));
          Ok(b.constant_bit32(ty, v.to_bits() as u32))
        }
        Const::F64(v) => {
          let ty = hint_ty.map(|t| ty_low.low(b, cre, t)).transpose()?.unwrap_or_else(|| b.type_float(64, None));
          Ok(b.constant_bit64(ty, v.to_bits() as u64))
        }
        Const::F128(_) => {
          Err("SPIR-V does not support 128-bit float constants".to_string())
        }
        Const::Array { .. } => {
          Err("SPIR-V: array constant lowering not yet implemented".to_string())
        }
        Const::Str(_) => {
          Err("SPIR-V: string literal constant lowering not yet implemented".to_string())
        }
      },
    }
  }
}
