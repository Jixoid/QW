/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Span;

use crate::{ExprId, FieldRng, Ident, ThingRng, TypeId, Visibility};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FieldKind {
  /// Member Variable Decl
  /// 
  /// e.g., `a: bool;`
  /// 
  /// To define multi define: `a,b,c: bool;`
  /// 
  MemberVar {kind: TypeId},


  /// Function Decl
  /// 
  /// e.g., `fun new(a: i32) {...}`
  /// 
  Fun {kind: TypeId, blok: Option<ExprId>},

  /// Constructor Decl
  /// 
  /// e.g., `init new(n_a: i32): a(n_a) {...}`
  /// 
  /// To define default constructor: `init() {...}`
  /// 
  Init {kind: TypeId, blok: Option<ExprId>, ils: ThingRng},
  
  /// Destructor Decl
  /// 
  /// e.g., `fini() {...}`
  /// 
  /// To define named destructor: `fini free() {...}`
  /// 
  Fini {kind: TypeId, blok: Option<ExprId>},


  /// Impl in Decl
  /// 
  /// This declaration is surreptitiously carried over to the global arena as a side effect.
  /// 
  /// e.g.,
  /// ```ignore
  /// impl: fmt::Display {
  ///   ...
  /// }
  /// ```
  /// 
  ImplIn {trait_ty: TypeId, ctn: FieldRng},


  /// Trait type Decl
  /// 
  /// e.g., `type Output;`
  /// 
  /// To override type: `type Output = i32;`
  /// 
  Type (Option<TypeId>),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Field {
  pub pos: Span,
  pub vis: Visibility,
  pub name: Option<Ident>,
  pub kind: FieldKind,
}
