/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{DefPath, DefPathId, Deps, Krate};
use qwc_string_interner::StrInterner;

use crate::Mangler;


pub struct ManglerBare;

impl Mangler for ManglerBare {
  fn new(cre: &Krate, deps: &Deps, sin: &StrInterner, id: DefPathId) -> String {
    let target_cre = if id.cid() == cre.cid() {
      cre
    } else {
      deps.get(id.cid())
    };
    let it = *target_cre.get(id);

    match it {
      DefPath::Root(name) => sin.str(name).to_string(),
      DefPath::Path {name, ..} => sin.str(name).to_string(),
      DefPath::Impl {base, ..} => Self::new(cre, deps, sin, base),
      DefPath::Spec {base, ..} => Self::new(cre, deps, sin, base),
      DefPath::AnonymousTuple(..) => String::from("tuple"),
    }
  }
}
