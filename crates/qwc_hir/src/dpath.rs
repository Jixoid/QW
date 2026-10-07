/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::num::NonZeroU32;

use thin_vec::ThinVec;


pub struct DPath(pub ThinVec<NonZeroU32>);

impl DPath {
  
  pub fn new(vec: Vec<NonZeroU32>) -> Self {
    Self(ThinVec::from(vec))
  }

}
