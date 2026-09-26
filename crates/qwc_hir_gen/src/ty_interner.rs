use qwc_hir::{Krate, Layout, LayoutBy, Type, TypeId, TypeKind};
use rustc_hash::FxHashMap;


pub struct TypeInterner {
  ty_generic_type: TypeId,
  
  ty_unit: TypeId,
  ty_never: TypeId,

  ty_bool: TypeId,

  // Int
  ty_isize: TypeId,
  ty_usize: TypeId,
  
  ty_i8: TypeId,
  ty_i16: TypeId,
  ty_i32: TypeId,
  ty_i64: TypeId,
  ty_i128: TypeId,
  
  ty_u8: TypeId,
  ty_u16: TypeId,
  ty_u32: TypeId,
  ty_u64: TypeId,
  ty_u128: TypeId,
  
  // Sub
  ty_ref: FxHashMap<(TypeId, bool), TypeId>,

  ty_option: FxHashMap<TypeId, TypeId>,
}


impl TypeInterner {
  pub fn new(cre: &mut Krate) -> Self {
    Self {
      ty_generic_type: cre.push(Type{kind: TypeKind::GenericType, layout: Layout::new_static(LayoutBy::QW)}),
      
      ty_unit:  cre.push(Type{kind: TypeKind::Unit, layout: Layout::new_static(LayoutBy::QW)}),
      ty_never: cre.push(Type{kind: TypeKind::Never, layout: Layout::new_inhabited(LayoutBy::QW)}),

      ty_bool: cre.push(Type{kind: TypeKind::Bool, layout: Layout::new_static(LayoutBy::QW)}),
      
      // Int
      ty_isize: cre.push(Type{kind: TypeKind::ArchInt(true), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_usize: cre.push(Type{kind: TypeKind::ArchInt(false), layout: Layout::new_static(LayoutBy::SYS)}),
      
      ty_i8:   cre.push(Type{kind: TypeKind::Int(8, true), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_i16:  cre.push(Type{kind: TypeKind::Int(16, true), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_i32:  cre.push(Type{kind: TypeKind::Int(32, true), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_i64:  cre.push(Type{kind: TypeKind::Int(64, true), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_i128: cre.push(Type{kind: TypeKind::Int(128, true), layout: Layout::new_static(LayoutBy::SYS)}),

      ty_u8:   cre.push(Type{kind: TypeKind::Int(8, false), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_u16:  cre.push(Type{kind: TypeKind::Int(16, false), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_u32:  cre.push(Type{kind: TypeKind::Int(32, false), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_u64:  cre.push(Type{kind: TypeKind::Int(64, false), layout: Layout::new_static(LayoutBy::SYS)}),
      ty_u128: cre.push(Type{kind: TypeKind::Int(128, false), layout: Layout::new_static(LayoutBy::SYS)}),

      // Sub
      ty_ref: FxHashMap::default(),
      ty_option: FxHashMap::default(),
    }
  }

  pub fn ty_generic_type(&self) -> TypeId { self.ty_generic_type }

  pub fn ty_unit(&self) -> TypeId  { self.ty_unit }
  pub fn ty_never(&self) -> TypeId { self.ty_never }

  pub fn ty_bool(&self) -> TypeId { self.ty_bool }

  // Int
  pub fn ty_isize(&self) -> TypeId { self.ty_isize }
  pub fn ty_usize(&self) -> TypeId { self.ty_usize }
  
  pub fn ty_i8(&self) -> TypeId   { self.ty_i8 }
  pub fn ty_i16(&self) -> TypeId  { self.ty_i16 }
  pub fn ty_i32(&self) -> TypeId  { self.ty_i32 }
  pub fn ty_i64(&self) -> TypeId  { self.ty_i64 }
  pub fn ty_i128(&self) -> TypeId { self.ty_i128 }

  pub fn ty_u8(&self) -> TypeId   { self.ty_u8 }
  pub fn ty_u16(&self) -> TypeId  { self.ty_u16 }
  pub fn ty_u32(&self) -> TypeId  { self.ty_u32 }
  pub fn ty_u64(&self) -> TypeId  { self.ty_u64 }
  pub fn ty_u128(&self) -> TypeId { self.ty_u128 }

  // Sub
  pub fn ty_ref(&mut self, cre: &mut Krate, id: TypeId, ism: bool) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_ref.entry((id, ism)) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let layout = (cre.get(id) as &Type).layout;

        let this = Type{
          kind: TypeKind::Ref(id, ism),
          layout,
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

  pub fn ty_option(&mut self, cre: &mut Krate, id: TypeId) -> TypeId {
    use std::collections::hash_map::Entry;
    
    match self.ty_option.entry(id) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let layout = (cre.get(id) as &Type).layout;

        let this = Type{
          kind: TypeKind::Option(id),
          layout,
        };

        let id = cre.push(this);

        entry.insert(id);
        id
      }
    }
  }

}
