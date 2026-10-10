/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_string_interner::Sid;

use crate::DefPathId;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DefPath {
  Root(Sid),
  Path{base: DefPathId, name: Sid},
  Impl{base: DefPathId, spec: DefPathId},
  Spec{base: DefPathId, spec: DefPathId},
}
