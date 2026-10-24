/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{self as ast, Attribute};
use qwc_diagnostic::{Label, Message, Status, Summary, msg::*, ret_err};
use qwc_hir::{self as hir, TypeAttr};

use crate::Ctx;


pub fn low_defpath(ctx: &mut Ctx, path: ast::DefPathId) -> hir::DefPathId {
  match *ctx.src.get(path) {
    ast::DefPath::Root(name) => ctx.cre.push(hir::DefPath::Root(name)),
    ast::DefPath::Path { base, name } => {
      let base = low_defpath(ctx, base);
      ctx.cre.push(hir::DefPath::Path { base, name })
    }
    ast::DefPath::Impl(base) => low_defpath(ctx, base),
    ast::DefPath::Generic(base) => low_defpath(ctx, base),
    ast::DefPath::Use(base) => low_defpath(ctx, base),
  }
}

pub fn read_attrs(ctx: &mut Ctx, sum: &mut Summary, attrs: Option<&Vec<Attribute>>) -> Status<TypeAttr> {
  let mut attr = TypeAttr::empty();

  if let Some(attrs) = attrs {
    for key in attrs {
      let key = key.ident;
      
      match () {
        // C
        _ if key.sid() == ctx.sin.sid_c() => {
          if attr.contains(TypeAttr::C) {
            ret_err!(sum, Message::error(DUPLICATE_ATTRIBUTE, Label::new(key, DEFINED_HERE))
              //+ Label::new(pos, FIRST_DEFINITION_HERE)
              + ONLY_ONE_DEFINITION_REMAIN
            );
          };
          attr |= TypeAttr::C;
        }

        _ => ret_err!(sum, Message::error(UNKNOWN_ATTRIBUTE.args(&[key.str(ctx.far)]), Label::new_pos(key)))
      }
    }
  }

  Status::Success(attr)
}
