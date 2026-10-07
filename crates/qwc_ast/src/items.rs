/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_diagnostic::Span;

use crate::{ExprId, FieldRng, ItemRng, ThingRng, TypeId, ident::Ident};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Visibility { Inherited, Public, Private, Protected, Crate, Super, Group }


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ItemKind {
  /// Global Variable Decl
  /// 
  /// To define constant value: `let pi: f32 = 3.14;`
  /// 
  /// To define variable: `var a: bool = true;`
  /// 
  Let {kind: Option<TypeId>, value: ExprId, ism: bool},


  /// Function Decl
  /// 
  /// To define static function: `fun main() {...}`
  /// 
  Fun {kind: TypeId, blok: Option<ExprId>},


  /// Task Decl
  /// 
  /// To define static task: `task do_it() {...}`
  /// 
  Task {kind: TypeId, blok: Option<ExprId>},
  

  /// Using Decl
  /// 
  /// e.g., `using IVec = std::Vec<i32>;`
  /// 
  Using (TypeId),
  
  /// Hidden Using Decl
  /// 
  /// e.g., `struct A {...}`
  /// 
  ItemTy (TypeId),
  

  // Module
  Krate (ItemRng),
  Module (ItemRng),
  ModuleUnloaded,
  ModuleFile (ItemRng, u16 /* fid */),
  
  // Generic
  Generic {params: ThingRng /* Name | NamedType */, reqs: ThingRng /* NamedTypeList */, ctn: ItemRng},
  
  // Impl
  Impl {type_ty: TypeId, trait_ty: Option<TypeId>, ctn: FieldRng},
  
  // Import
  Import (ThingRng),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Item {
  pub pos: Span,
  pub vis: Visibility,
  pub name: Option<Ident>,
  pub kind: ItemKind,
}
