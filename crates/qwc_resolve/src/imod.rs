/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_string_interner::Sid;
use crate::ExportMap;


#[derive(Clone, Copy)]
pub struct Imod<'a> {
  pub name: Sid,
  pub expmap: &'a ExportMap,
}

impl<'a> Imod<'a> {
  pub fn new(name: Sid, expmap: &'a ExportMap) -> Self {
    Self { name, expmap }
  }
}
