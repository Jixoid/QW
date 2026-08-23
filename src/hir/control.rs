use std::{ptr, ops::Range};

use crate::hir::{HirId, HirKind, Item, ItemId, Thing, ThingId, Type, TypeId};


pub type Rng = Range<u32>;


pub struct Crate<'a> {
  pub root: Option<ItemId>,
  
  pub imod: Vec<&'a Crate<'a>>,

  pub list_type: Vec<Type>,
  pub list_item: Vec<Item>,
  pub list_thing: Vec<Thing>,

  pub extra_data: Vec<ItemId>,
}


impl<'a> Crate<'a> {

  pub fn new() -> Self {
    Self{
      root: None,

      imod: vec![],

      list_type: vec![],
      list_item: vec![],
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

  pub fn new_thing(&mut self, v: Thing) -> ThingId {
    self.list_thing.push(v);

    let idx = self.list_thing.len() as u32 - 1;

    return HirId::new(HirKind::Thing, 0, idx);
  }


  pub fn get_extra(&self, rng: &Rng) -> &[ItemId] {
    let r = (rng.start as usize)..(rng.end as usize);

    &self.extra_data[r]
  }

  pub fn new_extra(&mut self, v: Vec<ItemId>) -> Rng {
    let start = self.extra_data.len() as u32;

    self.extra_data.extend(v);

    let end = self.extra_data.len() as u32;

    start..end
  }


  pub fn storage_size(&self) -> usize {
    size_of::<Type>()*self.list_type.len()
    +
    size_of::<Item>()*self.list_item.len()
    +
    size_of::<Thing>()*self.list_thing.len()
    +
    size_of::<ItemId>()*self.extra_data.len()
  }

}
