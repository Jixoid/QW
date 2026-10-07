/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_mir as mir;


pub enum Optimization {
  None = 0,
  Less = 1,
  Default = 2,
  Aggressive = 3,
}

pub enum OutKind {
  Object, ByteCode
}


pub trait ICGen {
  fn generate(&self, mir: &mir::Krate, ext_ll: bool, outk: OutKind, triple: &Option<String>, opt: Optimization) -> Result<(Vec<u8>, Option<String>), String>;

  fn run_vm(&self, code: &[u8]) -> Result<i32, String>;
}
