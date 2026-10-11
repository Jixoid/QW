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


pub struct ManglerQW;

impl Mangler for ManglerQW {
  fn new(cre: &Krate, deps: &Deps, sin: &StrInterner, id: DefPathId) -> String {
    format!("qw_{}", read(cre, deps, sin, id))
  }
}


fn read(cre: &Krate, deps: &Deps, sin: &StrInterner, id: DefPathId) -> String {
  let target_cre = if id.cid() == cre.cid() {
    cre
  } else {
    deps.get(id.cid())
  };
  let it = *target_cre.get(id);

  match it {
    DefPath::Root(name) => {
      let str = sin.str(name);
      format!("{}{}", str.len(), str)
    }
    
    DefPath::Path{base, name} => {
      let str = sin.str(name);
      let str = format!("{}{}", str.len(), str);
    
      format!("{}{}", read(cre, deps, sin, base), str)
    }
    
    DefPath::Impl{base, spec} => {
      format!("_impl_{}_{}", read(cre, deps, sin, base), read(cre, deps, sin, spec))
    }
    
    DefPath::Spec{base, spec} => {
      format!("_spec_{}_{}", read(cre, deps, sin, base), read(cre, deps, sin, spec))
    }

    DefPath::AnonymousTuple(rng) => {
      let parts = target_cre.extra_get(rng).map(|id| read(cre, deps, sin, id)).collect::<Vec<_>>();
      format!("_tup_{}_{}", parts.len(), parts.join("_"))
    }
  }
}
