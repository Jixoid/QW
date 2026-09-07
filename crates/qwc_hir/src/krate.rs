use core::slice;

use qwc_arena::Arena;

use crate::{AnyId, Expr, Item, ItemId, Type, TypeId, id::{HirId, HirKind, ExprId, NodeKind, SpecAny}, ty_interner::TypeInterner};


pub struct Krate {
  // Root
  root: Option<ItemId>,

  // Arena
  pub(super) list_type: Arena<Type>,
  list_expr: Arena<Expr>,
  list_item: Arena<Item>,

  extra_data: (Arena<HirId<SpecAny>>, Arena<NodeKind>),

  // Type Interning
  pub(super) tyin: Option<TypeInterner>,
}

impl Krate {

  pub fn new() -> Self {
    let mut ret = Self{
      root: None,
      
      list_type: Arena::new(),
      list_expr: Arena::new(),
      list_item: Arena::new(),
      
      extra_data: (Arena::new(), Arena::new()),

      tyin: None,
    };

    ret.tyin = Some(TypeInterner::new(&mut ret));

    ret
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
  pub fn extra<T: HirKind>(&mut self, vec: &[HirId<T>]) -> Rng {
    let (ids, kds) = &mut self.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const HirId<SpecAny>, vec.len()) };

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

  pub fn extra_get(&self, rng: Rng) -> impl Iterator<Item = (HirId<SpecAny>, NodeKind)> {
    let (ids, kinds) = &self.extra_data;
    let range = (rng.0 as usize)..(rng.1 as usize);

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| (id, kind))
  }
  

  // Size
  pub fn size_used<T: SizeApi>(&self) -> usize { T::size_used(&self) }
  pub fn size_alloc<T: SizeApi>(&self) -> usize { T::size_alloc(&self) }
  
  pub fn size_all_used(&self) -> usize {
    Self::size_used::<Type>(&self) + Self::size_used::<Expr>(&self) + Self::size_used::<Item>(&self) + Self::size_used::<AnyId>(&self)
  }

  pub fn size_all_alloc(&self) -> usize {
    Self::size_alloc::<Type>(&self) + Self::size_alloc::<Expr>(&self) + Self::size_alloc::<Item>(&self) + Self::size_alloc::<AnyId>(&self)
  }

}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Rng(pub u32, pub u32);

impl Rng {
  pub fn empty() -> Self { Self(0,0) }
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

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(0, u32::try_from(krate.list_type.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_type[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_type[id.idx() as usize] }
}

impl ArenaNode for Expr {
  type Id = ExprId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(0, u32::try_from(krate.list_expr.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_expr[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_expr[id.idx() as usize] }
}

impl ArenaNode for Item {
  type Id = ItemId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(0, u32::try_from(krate.list_item.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_item[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_item[id.idx() as usize] }
}


// size api
pub trait SizeApi {
  fn size_used(cre: &Krate) -> usize;
  fn size_alloc(cre: &Krate) -> usize;
}

impl SizeApi for Type {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_type.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_type.allocated_len() }
}

impl SizeApi for Expr {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_expr.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_expr.allocated_len() }
}

impl SizeApi for Item {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_item.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_item.allocated_len() }
}

impl SizeApi for AnyId {
  fn size_used(cre: &Krate) -> usize { (size_of::<HirId<SpecAny>>() * cre.extra_data.0.len()) + (size_of::<NodeKind>() * cre.extra_data.1.len()) }
  fn size_alloc(cre: &Krate) -> usize { (size_of::<HirId<SpecAny>>() * cre.extra_data.0.allocated_len()) + (size_of::<NodeKind>() * cre.extra_data.1.allocated_len()) }
}
