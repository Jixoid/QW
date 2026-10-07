/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{self as hir, Const};


pub trait Evaluator {
  fn evaluate(self, hir: &hir::Krate) -> Option<Const>;
}


impl Evaluator for hir::ExprId {
  fn evaluate(self, hir: &hir::Krate) -> Option<Const> {
    let it: &hir::Expr = hir.get(self);

    let val = match it.kind {
      hir::ExprKind::Const(v) => v,

      _ => todo!("cannot evaluate yet")
    };

    Some(val)
  }
}
