use core::slice;
use std::collections::HashMap;

use qwc_arena::{Arena, Files};
use qwc_diagnostic::{Message, msg};
use qwc_lexer::{WK, Word};

use crate::{AnyId, Attribute, Expr, ExprId, Ident, Item, ItemId, Patt, PattId, Thing, ThingId, Type, TypeId, id::{AstId, AstKind, NodeKind, SpecAny}, StrInterner};


pub struct Krate {
  // Root
  root: Option<ItemId>,

  // Arena
  list_type: Arena<Type>,
  list_expr: Arena<Expr>,
  list_item: Arena<Item>,
  list_patt: Arena<Patt>,
  list_thig: Arena<Thing>,

  extra_data: (Arena<AstId<SpecAny>>, Arena<NodeKind>),

  // Attribute
  map_attr: HashMap<AnyId, Vec<Attribute>>,
}


impl Krate {

  pub fn new() -> Self {
    Self{
      root: None,
      
      list_type: Arena::new(),
      list_expr: Arena::new(),
      list_item: Arena::new(),
      list_patt: Arena::new(),
      list_thig: Arena::new(),
      
      extra_data: (Arena::new(), Arena::new()),
      
      map_attr: HashMap::new()
    }
  }


  // Root
  pub fn root(&self) -> Option<ItemId> {
    self.root
  }

  pub fn set_root(&mut self, id: ItemId) {
    self.root = Some(id)
  }


  // Arena
  pub fn push<T: ArenaNode>(&mut self, obj: T) -> T::Id { T::push(self, obj) }
  pub fn get<T: ArenaNode>(&self, id: T::Id) -> &T { T::get(self, id) }
  pub fn get_mut<T: ArenaNode>(&mut self, id: T::Id) -> &mut T { T::get_mut(self, id) }


  // Extra
  pub fn extra<T: AstKind>(&mut self, vec: &[AstId<T>]) -> Rng {
    let (ids, kds) = &mut self.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const AstId<SpecAny>, vec.len()) };

    let irng = ids.extend_from_slice(vec);
    let krng = kds.extend_fill(T::kind(), vec.len());
    debug_assert_eq!(irng, krng);

    Rng(u32::try_from(irng.start).unwrap(), u32::try_from(irng.end).unwrap())
  }

  pub fn extra_any(&mut self, vec: &[AnyId]) -> Rng {
    let (ids, kds) = &mut self.extra_data;
    let start = ids.len();
    for any in vec {
      ids.push(any.id());
      kds.push(any.kind());
    }
    Rng(u32::try_from(start).unwrap(), u32::try_from(ids.len()).unwrap())
  }

  pub fn extra_get(&self, rng: Rng) -> impl Iterator<Item = (AstId<SpecAny>, NodeKind)> {
    let (ids, kinds) = &self.extra_data;
    let range = (rng.0 as usize)..(rng.1 as usize);

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| (id, kind))
  }


  // Attach
  pub fn attach<T: AttachNode>(&mut self, id: impl Into<AnyId>, obj: T) { T::attach(self, id.into(), obj); }


  // Size
  pub fn size(&self) -> usize {
    fn byte_size<T: Copy>(arena: &Arena<T>) -> usize { size_of::<T>() * arena.len() }
    
    byte_size(&self.list_type)
    +
    byte_size(&self.list_expr)
    +
    byte_size(&self.list_item)
    +
    byte_size(&self.list_patt)
    +
    byte_size(&self.list_thig)
    +
    byte_size(&self.extra_data.0)
    +
    byte_size(&self.extra_data.1)
  }
}


#[derive(Copy, Clone)]
pub struct Rng(pub u32, pub u32);

impl Rng {
  pub fn empty() -> Self { Self(0,0) }
}



// Ident save
pub trait IdentSave {
  fn ident(self, sin: &mut StrInterner, far: &Files) -> Result<Ident, Message>;
}

impl IdentSave for Word {
  fn ident(self, sin: &mut StrInterner, far: &Files) -> Result<Ident, Message> {
    let (off, len, fid, kind) = self.to();
    
    let str = str::from_utf8(&far.get(fid).map()[(off as usize)..((off as usize)+(len.get() as usize))]).unwrap();

    if kind != WK::Word { return Err(Message::error(self, msg::EXPECTED_IDENTIFIER, &[ str ])) }
  
    let sid = sin.sid(str);

    Ok(Ident::new(off, len, fid, sid))
  }
}


// push & get
pub trait ArenaNode {
  type Id;
  
  fn push(krate: &mut Krate, obj: Self) -> Self::Id;
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self;
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self;
}

impl ArenaNode for Type {
  type Id = TypeId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_type.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_type[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_type[id.idx() as usize] }
}

impl ArenaNode for Expr {
  type Id = ExprId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_expr.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_expr[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_expr[id.idx() as usize] }
}

impl ArenaNode for Item {
  type Id = ItemId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_item.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_item[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_item[id.idx() as usize] }
}

impl ArenaNode for Patt {
  type Id = PattId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_patt.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_patt[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_patt[id.idx() as usize] }
}

impl ArenaNode for Thing {
  type Id = ThingId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_thig.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_thig[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_thig[id.idx() as usize] }
}


// attach
pub trait AttachNode {
  fn attach(krate: &mut Krate, id: AnyId, obj: Self);
}

impl AttachNode for Vec<Attribute> {
  fn attach(krate: &mut Krate, id: AnyId, obj: Self) { krate.map_attr.insert(id, obj); }
}
