/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_string_interner::Sid;

use crate::{Const, ExprId, TypeId};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Thing {
  NamedType(Sid, TypeId),
  NamedExpr(Sid, ExprId),
  NamedConst(Sid, Const),
}
