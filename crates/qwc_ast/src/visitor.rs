/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_arena::Files;
use qwc_diagnostic::Summary;
use qwc_string_interner::StrInterner;

use crate::Krate;


pub trait Visitor {
  type Type;

  fn visit(cre: &Krate, sin: &StrInterner, far: &Files) -> Result<Self::Type, Summary>;
}
