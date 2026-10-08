/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use rustc_hash::FxHashMap;
use inkwell::values::BasicValueEnum;
use qwc_mir::SSA;


pub struct FunCtx<'ctx> {
  pub params: Vec<BasicValueEnum<'ctx>>,
  pub ssa_map: FxHashMap<SSA, BasicValueEnum<'ctx>>,
}


impl<'ctx> FunCtx<'ctx> {
  pub fn new(params: Vec<BasicValueEnum<'ctx>>) -> Self {
    Self {
      params,
      ssa_map: FxHashMap::default(),
    }
  }
}

