/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Poit;
use qwc_string_interner::StrInterner;

use crate::{DefPath, DefPathId, Deps, Krate};


pub struct HPoit {
  str: String,
}

impl HPoit {
  pub fn new(cre: &Krate, ideps: &Deps, sin: &StrInterner, path: DefPathId) -> Self {
    let str = read(cre, ideps, sin, path);

    Self { str }
  }
}

impl Poit for HPoit {
  fn to_str(&self) -> &str { &self.str }
  fn to_string(self) -> String { self.str }
}

impl std::fmt::Display for HPoit {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.str)
  }
}


fn read(cre: &Krate, ideps: &Deps, sin: &StrInterner, id: DefPathId) -> String {
  let it = *if id.cid() == cre.cid() { cre } else { ideps.get(id.cid()) }.get(id);

  match it {
    DefPath::Root(name) => sin.str(name).to_string(),
    
    DefPath::Path{base, name} => format!("{}::{}", read(cre, ideps, sin, base), sin.str(name)),
    
    DefPath::Impl{base, spec} => format!("<impl {} for {}>", read(cre, ideps, sin, base), read(cre, ideps, sin, spec)),
    
    DefPath::Spec{base, spec} => format!("{}<{}>", read(cre, ideps, sin, base), read(cre, ideps, sin, spec)),

    DefPath::AnonymousTuple(rng) => {
      let items = cre.extra_get(rng).map(|id| read(cre, ideps, sin, id)).collect::<Vec<_>>();
      format!("({})", items.join(", "))
    }
  }
}