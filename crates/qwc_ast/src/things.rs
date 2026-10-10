/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Span;

use crate::{ExprId, ExprRng, Ident, ThingId, ThingRng, TypeId, TypeRng, Visibility};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ThingKind {
  Name(Ident),
  List(ThingRng),
  
  Wildcard, Crate, Super,
  
  NamedExpr(Ident, ExprId),
  NamedType(Ident, TypeId),
  
  Alias(ThingId, Ident),
  
  TypeVis(TypeId, Visibility),
  
  NamedTypeVis(Ident, Visibility, TypeId),
  
  NamedTypeList(Ident, TypeRng),
  NamedExprList(Ident, ExprRng),
  
  MatchArm(ExprId /* pat */, ExprId /* body */),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Thing {
  pub kind: ThingKind,
  pub pos: Span,
}
