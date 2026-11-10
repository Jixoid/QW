/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


mod type_p;
mod value_p;
mod inst_p;
mod symb_p;
mod cgen;


pub use cgen::GenSpirV as Gen;

#[cfg(test)]
mod tests {
  use rspirv::binary::{Assemble, Disassemble};
  use rspirv::dr::Builder;
  use rspirv::spirv;

  #[test]
  fn test_rspirv_basic() {
    let mut b = Builder::new();
    b.capability(spirv::Capability::Shader);
    b.memory_model(spirv::AddressingModel::Logical, spirv::MemoryModel::GLSL450);

    let i32_ty = b.type_int(32, 1);
    let fn_ty = b.type_function(i32_ty, vec![i32_ty]);
    let fn_id = b.begin_function(i32_ty, None, spirv::FunctionControl::NONE, fn_ty).unwrap();
    let param_id = b.function_parameter(i32_ty).unwrap();
    b.begin_block(None).unwrap();
    let const_10 = b.constant_bit32(i32_ty, 10);
    let add_res = b.i_add(i32_ty, None, param_id, const_10).unwrap();
    b.ret_value(add_res).unwrap();
    b.end_function().unwrap();
    b.entry_point(spirv::ExecutionModel::GLCompute, fn_id, "add10", vec![]);

    let module = b.module();
    let text = module.disassemble();
    println!("{}", text);
    let words = module.assemble();
    assert!(!words.is_empty());
    assert_eq!(words[0], 0x07230203);
  }
}
