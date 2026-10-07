/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{Ident, PattRng};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Patt {
  One(Ident),
  
  /// _
  Under,
  /// ..
  Rest,
  
  Tuple(PattRng),
  Array(PattRng),
}
