use std::{ptr, ops::Range};

use crate::hir::{AnyId, Expr, ExprId, HirId, HirKind, Item, ItemId, Thing, ThingId, Type, TypeId};


pub type Rng = Range<u32>;


pub struct Crate<'a> {
  pub root: Option<ItemId>,
  
  pub imod: Vec<&'a Crate<'a>>,

  pub list_type: Vec<Type>,
  pub list_item: Vec<Item>,
  pub list_expr: Vec<Expr>,
  pub list_thing: Vec<Thing>,

  pub extra_data: Vec<AnyId>,
}


impl<'a> Crate<'a> {

  pub fn new() -> Self {
    Self{
      root: None,

      imod: vec![],

      list_type: vec![],
      list_item: vec![],
      list_expr: vec![],
      list_thing: vec![],

      extra_data: vec![],
    }
  }


  pub fn add_dep(&mut self, mol: &'a Self) -> u16 {
    match self.imod.iter().position(|&x| ptr::eq(x, mol)) {
      Some(r) => (r as u16) +1,
      None => {
        self.imod.push(mol);
        self.imod.len() as u16
      }
    }
  }

  pub fn get_dep(&self, mod_id: u16) -> &'a Self {
    self.imod[(mod_id -1) as usize] 
  }


  pub fn localize<T>(&mut self, mol: &'a Self, eid: HirId<T>) -> HirId<T> {
    let omol = if eid.krate == 0 { mol } else { mol.get_dep(eid.krate) };

    let lmid = self.add_dep(omol);

    HirId::new(eid.kind, lmid, eid.index)
  }


  pub fn get_type(&self, id: TypeId) -> &Type {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_type[id.index as usize]
  }

  pub fn get_item(&self, id: ItemId) -> &Item {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_item[id.index as usize]
  }

  pub fn get_expr(&self, id: ExprId) -> &Expr {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_expr[id.index as usize]
  }

  pub fn get_thing(&self, id: ThingId) -> &Thing {
    let modl: &Crate<'a> = if id.krate == 0 { self } else { &self.imod[(id.krate-1) as usize] };

    &modl.list_thing[id.index as usize]
  }


  pub fn new_type(&mut self, v: Type) -> TypeId {
    self.list_type.push(v);

    let idx = self.list_type.len() as u32 - 1;

    return HirId::new(HirKind::Type, 0, idx);
  }

  pub fn new_item(&mut self, v: Item) -> ItemId {
    self.list_item.push(v);

    let idx = self.list_item.len() as u32 - 1;

    return HirId::new(HirKind::Item, 0, idx);
  }

  pub fn new_expr(&mut self, v: Expr) -> ExprId {
    self.list_expr.push(v);

    let idx = self.list_expr.len() as u32 - 1;

    return HirId::new(HirKind::Expr, 0, idx);
  }

  pub fn new_thing(&mut self, v: Thing) -> ThingId {
    self.list_thing.push(v);

    let idx = self.list_thing.len() as u32 - 1;

    return HirId::new(HirKind::Thing, 0, idx);
  }


  pub fn get_extra(&self, rng: &Rng) -> &[AnyId] {
    let r = (rng.start as usize)..(rng.end as usize);

    &self.extra_data[r]
  }

  pub fn new_extra(&mut self, v: Vec<AnyId>) -> Rng {
    let start = self.extra_data.len() as u32;

    self.extra_data.extend(v);

    let end = self.extra_data.len() as u32;

    start..end
  }

  pub fn new_extra_from<T>(&mut self, v: &[HirId<T>]) -> Rng {
    let start = self.extra_data.len() as u32;

    for x in v {
      self.extra_data.push(x.to_any());
    }

    let end = self.extra_data.len() as u32;

    start..end
  }


  pub fn is_same_type(&self, t1: TypeId, t2: TypeId) -> bool {
    if t1 == t2 { return true; }

    let ty1 = self.get_type(t1);
    let ty2 = self.get_type(t2);

    match (ty1, ty2) {
      (Type::Fun { args: a1, ret: r1 }, Type::Fun { args: a2, ret: r2 }) => {
        let args1 = self.get_extra(a1);
        let args2 = self.get_extra(a2);
        if args1.len() != args2.len() { return false; }
        for (x1, x2) in args1.iter().zip(args2.iter()) {
          let th1 = self.get_thing(ThingId::new_from(*x1));
          let th2 = self.get_thing(ThingId::new_from(*x2));
          match (th1, th2) {
            (Thing::NamedType(_, sub_ty1), Thing::NamedType(_, sub_ty2)) => {
              if !self.is_same_type(*sub_ty1, *sub_ty2) { return false; }
            }
            _ => return false,
          }
        }

        match (r1, r2) {
          (Some(sub1), Some(sub2)) => self.is_same_type(*sub1, *sub2),
          (None, None) => true,
          _ => false,
        }
      }

      (Type::Ref(sub1, acc1), Type::Ref(sub2, acc2)) => {
        acc1 == acc2 && self.is_same_type(*sub1, *sub2)
      }
      (Type::Ptr(sub1, acc1), Type::Ptr(sub2, acc2)) => {
        acc1 == acc2 && self.is_same_type(*sub1, *sub2)
      }
      (Type::DynArr(sub1), Type::DynArr(sub2)) => self.is_same_type(*sub1, *sub2),
      (Type::StaArr(sub1, s1), Type::StaArr(sub2, s2)) => s1 == s2 && self.is_same_type(*sub1, *sub2),
      (Type::Option(sub1), Type::Option(sub2)) => self.is_same_type(*sub1, *sub2),
      (Type::Result(s1, e1), Type::Result(s2, e2)) => self.is_same_type(*s1, *s2) && self.is_same_type(*e1, *e2),

      _ => ty1 == ty2,
    }
  }


  pub fn is_assignable(&self, src: TypeId, dest: TypeId) -> bool {
    if src == dest { return true }

    let ty_src = self.get_type(src);
    let ty_dest = self.get_type(dest);

    if ty_src == ty_dest { return true; }


    match (ty_src, ty_dest) {
      // Reference Coercion: &mut T -> &T (MUT to IMM)
      (Type::Ref(sub_s, crate::hir::AccessKind::MUT), Type::Ref(sub_d, crate::hir::AccessKind::IMM)) => {
        sub_s == sub_d || self.get_type(*sub_s) == self.get_type(*sub_d)
      }

      // Pointer Coercion: ^mut T -> ^T (MUT to IMM)
      (Type::Ptr(sub_s, crate::hir::AccessKind::MUT), Type::Ptr(sub_d, crate::hir::AccessKind::IMM)) => {
        sub_s == sub_d || self.get_type(*sub_s) == self.get_type(*sub_d)
      }

      // Option Wrapping: T -> Option<T>
      (_, Type::Option(sub_d)) => self.is_assignable(src, *sub_d),

      // Integer widening (must preserve signedness: smaller -> larger)
      (Type::Int(b1, s1), Type::Int(b2, s2)) => s1 == s2 && b1 <= b2,
      (Type::Int(b, s1), Type::ArchInt(s2)) => s1 == s2 && *b <= 64,
      (Type::ArchInt(s1), Type::Int(b, s2)) => s1 == s2 && *b >= 64,
      (Type::ArchInt(s1), Type::ArchInt(s2)) => s1 == s2,

      _ => false,
    }
  }


  pub fn storage_size(&self) -> usize {
    size_of::<Type>()*self.list_type.len()
    +
    size_of::<Item>()*self.list_item.len()
    +
    size_of::<Expr>()*self.list_expr.len()
    +
    size_of::<Thing>()*self.list_thing.len()
    +
    size_of::<AnyId>()*self.extra_data.len()
  }

}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::hir::AccessKind;

  #[test]
  fn test_type_equality_and_assignability() {
    let mut krate = Crate::new();

    let i32_ty = krate.new_type(Type::Int(32, true));
    let u32_ty = krate.new_type(Type::Int(32, false));
    let i32_alias = krate.new_type(Type::Int(32, true));

    // Eq trait equality
    assert_eq!(krate.get_type(i32_ty), krate.get_type(i32_alias));
    assert_ne!(krate.get_type(i32_ty), krate.get_type(u32_ty));

    // Pointer mutability coercion: ^mut i32 -> ^i32
    let ptr_mut = krate.new_type(Type::Ptr(i32_ty, AccessKind::MUT));
    let ptr_imm = krate.new_type(Type::Ptr(i32_ty, AccessKind::IMM));
    assert!(krate.is_assignable(ptr_mut, ptr_imm));
    assert!(!krate.is_assignable(ptr_imm, ptr_mut));

    // Reference mutability coercion: &mut i32 -> &i32
    let ref_mut = krate.new_type(Type::Ref(i32_ty, AccessKind::MUT));
    let ref_imm = krate.new_type(Type::Ref(i32_ty, AccessKind::IMM));
    assert!(krate.is_assignable(ref_mut, ref_imm));
    assert!(!krate.is_assignable(ref_imm, ref_mut));

    // Option wrapping: i32 -> Option<i32>
    let opt_i32 = krate.new_type(Type::Option(i32_ty));
    let bool_ty = krate.new_type(Type::Bool);
    assert!(krate.is_assignable(i32_ty, opt_i32));
    assert!(!krate.is_assignable(bool_ty, opt_i32));
    assert!(!krate.is_assignable(u32_ty, opt_i32));
  }
}
