/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_cgen::{IBackend, ICodeGen, ILinkObj, IRunVM};


pub struct GenSpirV;

impl IBackend for GenSpirV {
	fn code_gen(&self) -> Box<dyn ICodeGen> { todo!() }
	fn link_obj(&self) -> Box<dyn ILinkObj> { todo!() }
	fn run_vm(&self) -> Option<Box<dyn IRunVM>> { None }
}
