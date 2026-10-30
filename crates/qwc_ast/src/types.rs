/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Span;

use crate::{AnyRng, DefPathId, ExprId, FieldRng, Ident, ThingRng, TypeId, TypeRng};



pub enum FunAttrs {
  Pure = 0x01,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
  // Must Resolve
  Nick (Ident),
  Path (TypeRng),

  // Pointer
  Ptr {kind: TypeId, ism: bool, addrspace: Option<(ExprId, Span)>},
  Ref {kind: TypeId, ism: bool, addrspace: Option<(ExprId, Span)>},
  
  /// Vector
  Vector (TypeId, ExprId),
  VScale (TypeId),

  // Sequential
  Array (TypeId, ExprId),
  Slice (TypeId),
  
  // Must Context
  Type,
  SelfT,
  
  // Basic
  Unit,
  Range (TypeId),

  // Variant
  Fail    (TypeId),
  Option  (TypeId),
  Result  {sub: TypeId, err: TypeId},
  Variant {fields: ThingRng /* Name | NamedType */, path: DefPathId},

  // Enum
  Enum  {fields: ThingRng /* Name | NamedExpr */, path: DefPathId},
  Flags {fields: ThingRng /* Name | NamedExpr */, path: DefPathId},

  // Combinated
  Struct   {vars: ThingRng /* NamedType => name: type */, path: DefPathId},
  Tuple    {vars: TypeRng, path: DefPathId},
  TupleAnon(TypeRng),

  // Impl
  Iface {fields: FieldRng, reqs: TypeRng, path: DefPathId},
  Trait {fields: FieldRng, reqs: TypeRng, path: DefPathId},

  // Function
  Fun {self_kind: Option<TypeId>, args: ThingRng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
  Task{self_kind: Option<TypeId>, args: ThingRng /* NamedType */, ret: Option<TypeId>, attr: u8 /* FunAttrs */},
  Init{args: ThingRng /* NamedType */, attr: u8 /* FunAttrs */},
  Fini{args: ThingRng /* NamedType */, attr: u8 /* FunAttrs */},

  // Specialize
  Spec{base: TypeId, args: AnyRng /* TypeId | ExprId */},
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type {
  pub pos: Span,
  pub kind: TypeKind,
}
