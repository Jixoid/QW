/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use thin_vec::ThinVec;

use crate::{ExprId, Ident};


#[derive(Clone)]
pub enum AttrKind {
  One(),
  Bin(Ident),
  Set(ExprId), 
  List(ThinVec<Attribute>),
}

#[derive(Clone)]
pub struct Attribute {
  pub ident: Ident,
  pub kind: AttrKind,
}
