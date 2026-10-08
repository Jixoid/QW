/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_hir::{Krate, Layout, LayoutBy, Type, TypeAttr, TypeId, TypeKind};
use rustc_hash::FxHashMap;


pub struct TypeInterner {
  ty_meta: FxHashMap<TypeId, TypeId>,
  
  ty_ref: FxHashMap<(TypeId, bool), TypeId>,
  
  ty_trait_from: FxHashMap<(TypeId, TypeId), TypeId>,
  
  ty_slice: FxHashMap<TypeId, TypeId>,
  ty_vscale: FxHashMap<TypeId, TypeId>,

  ty_option: FxHashMap<TypeId, TypeId>,
}


impl TypeInterner {
  pub fn new() -> Self {
    Self {
      ty_meta: FxHashMap::default(),
      ty_ref: FxHashMap::default(),
      ty_trait_from: FxHashMap::default(),
      ty_slice: FxHashMap::default(),
      ty_vscale: FxHashMap::default(),
      ty_option: FxHashMap::default(),
    }
  }

  // Sub
  pub fn ty_meta(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_meta.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Meta(id),
          layout: Layout::new_meta(LayoutBy::QW),
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_ref(&mut self, cre: &mut Krate, id: TypeId, ism: bool) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_ref.entry((id, ism)) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Ref(id, ism),
          layout: Layout::new_static(LayoutBy::QW),
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
          layout: Layout::new_static(LayoutBy::QW),
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
          layout: Layout::new_static(LayoutBy::QW),
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
          layout: Layout::new_static(LayoutBy::QW),
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_option(&mut self, cre: &mut Krate, id: TypeId, layout: Layout) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_option.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let this = Type{
          kind: TypeKind::Option(id),
          layout,
          attr: TypeAttr::empty(),
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

}
