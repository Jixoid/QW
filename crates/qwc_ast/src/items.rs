use qwc_diagnostic::Span;

use crate::{ExprId, Rng, TypeId, ident::Ident};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Visibility { Inherited, Public, Private, Protected, Crate, Super, Group }


#[derive(Debug, Copy, Clone)]
pub enum ItemKind {
  // Variable
  Let {kind: Option<TypeId>, value: ExprId, ism: bool},

  // Member
  Member {kind: TypeId},
  
  // Function
  Fun  {kind: TypeId, blok: Option<ExprId>},
  Init {kind: TypeId, blok: Option<ExprId>, ils: Rng /* NamedExprList */},
  Fini {kind: TypeId, blok: Option<ExprId>},
  
  // Using
  Using (TypeId),
  ItemTy (TypeId),
  
  // Module
  Krate (Rng /* ItemId */),
  Module (Rng /* ItemId */),
  ModuleUnloaded,
  ModuleFile (Rng /* ItemId */, u16 /* fid */),
  
  // Generic
  Generic {params: Rng /* Name | NamedType */, reqs: Rng /* NamedTypeList */, ctn: Rng /* ItemId */},
  
  // Impl
  Impl {type_ty: TypeId, trait_ty: Option<TypeId>, ctn: Rng /* ItemId */},
  ImplIn {trait_ty: TypeId, ctn: Rng /* ItemId */},
  
  // Import
  Import (Rng /* ThingId */),
}


#[derive(Debug, Copy, Clone)]
pub struct Item {
  pub pos: Span,
  pub vis: Visibility,
  pub name: Option<Ident>,
  pub kind: ItemKind,
}
