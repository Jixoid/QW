/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::fmt;
use qwc_mir as mir;


#[derive(Default)]
pub struct Module {
  str: String,
}

impl Module {
  pub fn stream(&mut self) -> &mut dyn fmt::Write {
    &mut self.str
  }

  pub fn finish(self) -> String {
    self.str
  }
}


pub fn low_type(cre: &mir::Krate, ty: mir::TypeId) -> Result<String, String> {
  match cre.get(ty).kind {
    mir::TypeKind::Unit => Ok("none".to_string()),
    mir::TypeKind::Bool => Ok("i1".to_string()),
    mir::TypeKind::Int(bits, _) => Ok(format!("i{}", bits.get())),

    ref other => Err(format!("unsupported type in CIRCT backend: {:?}", other)),
  }
}

pub fn low_val(val: &mir::Value) -> Result<String, String> {
  match val {
    mir::Value::SSA(ssa) => Ok(format!("%{}", ssa.get())),
    mir::Value::Param(idx) => Ok(format!("%arg{}", idx)),
    mir::Value::Const(c) => match c {
      mir::Const::Bool(b) => Ok(if *b { "true".to_string() } else { "false".to_string() }),
      mir::Const::I8(v) => Ok(format!("{}", *v as i8)),
      mir::Const::I16(v) => Ok(format!("{}", *v as i16)),
      mir::Const::I32(v) => Ok(format!("{}", *v as i32)),
      mir::Const::I64(v) => Ok(format!("{}", *v as i64)),
      mir::Const::I128(v) => Ok(format!("{}", *v as i128)),
      mir::Const::Unit => Ok("()".to_string()),

      _ => Err(format!("unsupported constant in CIRCT: {:?}", c)),
    },
    mir::Value::GlobalRef(..) => Err("global references not supported in CIRCT".to_string()),
  }
}
