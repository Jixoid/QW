/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{Deps, ExprId, GetApi, HirId, HirKind, ItemId, Krate, Layout, Type, TypeAttr, TypeId, TypeKind};
use rustc_hash::FxHashMap;


pub struct TypeInterner {
  ty_meta: FxHashMap<TypeId, TypeId>,
  
  ty_ref: FxHashMap<(TypeId, bool), TypeId>,
  ty_ptr: FxHashMap<(TypeId, bool), TypeId>,
  
  ty_trait_from: FxHashMap<(TypeId, TypeId), TypeId>,
  
  ty_slice: FxHashMap<TypeId, TypeId>,
  ty_vscale: FxHashMap<TypeId, TypeId>,
  
  ty_range: FxHashMap<TypeId, TypeId>,
  
  ty_option: FxHashMap<TypeId, TypeId>,

  ty_generic_type: FxHashMap<(ItemId, u32), TypeId>,
}


impl TypeInterner {
  pub fn new() -> Self {
    Self {
      ty_meta: FxHashMap::default(),
      ty_ref: FxHashMap::default(),
      ty_ptr: FxHashMap::default(),
      ty_trait_from: FxHashMap::default(),
      ty_slice: FxHashMap::default(),
      ty_vscale: FxHashMap::default(),
      ty_range: FxHashMap::default(),
      ty_option: FxHashMap::default(),
      ty_generic_type: FxHashMap::default(),
    }
  }

  fn get<'a, T: HirKind + GetApi<T>>(cre: &'a Krate, ideps: &'a Deps, id: HirId<T>) -> &'a T {
    if id.cid() == cre.cid() { cre.get(id) } else { ideps.get(id.cid()).get(id) }
  }

  
  pub fn ty_meta(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_meta.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Meta(id),
          layout: Layout::META,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_ref(&mut self, cre: &mut Krate, id: TypeId, ism: bool, addrspace: Option<ExprId>) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_ref.entry((id, ism)) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Ref{kind: id, ism, addrspace},
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_ptr(&mut self, cre: &mut Krate, id: TypeId, ism: bool, addrspace: Option<ExprId>) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_ptr.entry((id, ism)) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Ptr{kind: id, ism, addrspace},
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_trait_from(&mut self, cre: &mut Krate, id: TypeId, trait_ty: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_trait_from.entry((id, trait_ty)) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::TraitFrom{hidden: id, trait_ty},
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_slice(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_slice.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Slice(id),
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_vscale(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_vscale.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::VScale(id),
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_range(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_range.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Range(id),
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_option(&mut self, cre: &mut Krate, ideps: &Deps, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_option.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let it = Self::get(cre, ideps, id);
        let this = Type{
          kind: TypeKind::Option(id),
          layout: it.layout,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_generic_type(&mut self, cre: &mut Krate, generic: ItemId, idx: u32) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_generic_type.entry((generic, idx)) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::GenericType{generic, idx},
          layout: Layout::STATIC,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

}
