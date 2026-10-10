/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

use qwc_hir::{DefPath, DefPathId, Krate};
use qwc_string_interner::StrInterner;

use crate::Mangler;


pub struct ManglerQW;

impl Mangler for ManglerQW {
  fn new(cre: &Krate, sin: &StrInterner, id: DefPathId) -> String {
    format!("qw_{}", read(cre, sin, id))
  }
}


fn read(cre: &Krate, sin: &StrInterner, id: DefPathId) -> String {
  let it = *cre.get(id);

  match it {
    DefPath::Root(name) => {
      let str = sin.str(name);
      format!("{}{}", str.len(), str)
    }
    
    DefPath::Path{base, name} => {
      let str = sin.str(name);
      let str = format!("{}{}", str.len(), str);
    
      format!("{}{}", read(cre, sin, base), str)
    }
    
    DefPath::Impl{base, spec} => {
      format!("_impl_{}_{}", read(cre, sin, base), read(cre, sin, spec))
    }
    
    DefPath::Spec{base, spec} => {
      format!("_spec_{}_{}", read(cre, sin, base), read(cre, sin, spec))
    }
  }
}
