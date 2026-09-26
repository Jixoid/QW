use core::slice;
use rustc_hash::FxHashMap;

use qwc_arena::{Arena, Files};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::{WK, Word};
use qwc_string_interner::StrInterner;

use crate::{AnyId, AnyRng, Attribute, Expr, ExprId, Field, FieldId, Ident, Item, ItemId, Patt, PattId, Thing, ThingId, Type, TypeId, id::{AstId, AstKind, NodeKind, Rng, SpecAny}};


pub struct Krate {
  // Root
  root: Option<ItemId>,

  // Arena
  list_type: Arena<Type>,
  list_expr: Arena<Expr>,
  list_item: Arena<Item>,
  list_patt: Arena<Patt>,
  list_fiel: Arena<Field>,
  list_thig: Arena<Thing>,

  extra_data: (Arena<AstId<SpecAny>>, Arena<NodeKind>),

  // Attribute
  map_attr: FxHashMap<AnyId, Vec<Attribute>>,
}


impl Krate {

  pub fn new() -> Self {
    Self{
      root: None,
      
      list_type: Arena::new(),
      list_expr: Arena::new(),
      list_item: Arena::new(),
      list_patt: Arena::new(),
      list_fiel: Arena::new(),
      list_thig: Arena::new(),
      
      extra_data: (Arena::new(), Arena::new()),
      
      map_attr: FxHashMap::default(),
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
  pub fn extra<T: AstKind>(&mut self, vec: &[AstId<T>]) -> Rng<T> {
    let (ids, kds) = &mut self.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const AstId<SpecAny>, vec.len()) };

    let irng = ids.extend_from_slice(vec);
    let krng = kds.extend_fill(T::kind(), vec.len());
    debug_assert_eq!(irng, krng);

    Rng::new(u32::try_from(irng.start).unwrap(), u32::try_from(irng.end).unwrap())
  }

  pub fn extra_any(&mut self, vec: &[AnyId]) -> AnyRng {
    let (ids, kds) = &mut self.extra_data;
    let start = ids.len();
    
    for any in vec {
      ids.push(any.id());
      kds.push(any.kind());
    }
    
    AnyRng::new(u32::try_from(start).unwrap(), u32::try_from(ids.len()).unwrap())
  }

  pub fn extra_get<T: AstKind>(&self, rng: Rng<T>) -> impl Iterator<Item = AstId<T>> {
    let (ids, kinds) = &self.extra_data;
    let range = rng.range();

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| AstId::<T>::new_from((id, kind)))
  }

  pub fn extra_any_get(&self, rng: AnyRng) -> impl Iterator<Item = AnyId> {
    let (ids, kinds) = &self.extra_data;
    let range = rng.range();

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| AnyId::new_from((id, kind)))
  }


  // Attach
  pub fn attach<T: AttachNode>(&mut self, id: impl Into<AnyId>, obj: T) { T::attach(self, id.into(), obj); }
  pub fn get_attached<T: AttachNode>(&self, id: impl Into<AnyId>) -> Option<&T> { T::get(self, id.into()) }
  pub fn get_attached_mut<T: AttachNode>(&mut self, id: impl Into<AnyId>) -> Option<&mut T> { T::get_mut(self, id.into()) }


  // Size
  pub fn size_used<T: SizeApi>(&self) -> usize { T::size_used(&self) }
  pub fn size_alloc<T: SizeApi>(&self) -> usize { T::size_alloc(&self) }
  
  pub fn size_all_used(&self) -> usize {
    Self::size_used::<Type>(&self) + Self::size_used::<Expr>(&self) + Self::size_used::<Item>(&self) + Self::size_used::<Patt>(&self) + Self::size_used::<Field>(&self) + Self::size_used::<Thing>(&self) + Self::size_used::<AnyId>(&self)
  }

  pub fn size_all_alloc(&self) -> usize {
    Self::size_alloc::<Type>(&self) + Self::size_alloc::<Expr>(&self) + Self::size_alloc::<Item>(&self) + Self::size_alloc::<Patt>(&self) + Self::size_alloc::<Field>(&self) + Self::size_alloc::<Thing>(&self) + Self::size_alloc::<AnyId>(&self)
  }

}



// Ident save
pub trait IdentSave {
  fn ident(self, sin: &mut StrInterner, far: &Files) -> Result<Ident, Message>;
}

impl IdentSave for Word {
  fn ident(self, sin: &mut StrInterner, far: &Files) -> Result<Ident, Message> {
    let (off, len, fid, kind) = self.to();
    
    let str = str::from_utf8(&far.get(fid).map()[(off as usize)..((off as usize)+(len.get() as usize))]).unwrap();

    if kind != WK::Word { return Err(Message::error(EXPECTED_IDENTIFIER, Label::new_pos(self))) }
  
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

impl ArenaNode for Field {
  type Id = FieldId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_fiel.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_fiel[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_fiel[id.idx() as usize] }
}

impl ArenaNode for Thing {
  type Id = ThingId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_thig.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_thig[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_thig[id.idx() as usize] }
}


// attach
pub trait AttachNode: Sized {
  fn attach(krate: &mut Krate, id: AnyId, obj: Self);
  fn get<'a>(krate: &'a Krate, id: AnyId) -> Option<&'a Self>;
  fn get_mut<'a>(krate: &'a mut Krate, id: AnyId) -> Option<&'a mut Self>;
}

impl AttachNode for Vec<Attribute> {
  fn attach(krate: &mut Krate, id: AnyId, obj: Self) { krate.map_attr.insert(id, obj); }
  fn get<'a>(krate: &'a Krate, id: AnyId) -> Option<&'a Self> { krate.map_attr.get(&id) }
  fn get_mut<'a>(krate: &'a mut Krate, id: AnyId) -> Option<&'a mut Self> { krate.map_attr.get_mut(&id) }
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

impl SizeApi for Patt {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_patt.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_patt.allocated_len() }
}

impl SizeApi for Field {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_fiel.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_fiel.allocated_len() }
}

impl SizeApi for Thing {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_thig.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_thig.allocated_len() }
}

impl SizeApi for AnyId {
  fn size_used(cre: &Krate) -> usize { (size_of::<AstId<SpecAny>>() * cre.extra_data.0.len()) + (size_of::<NodeKind>() * cre.extra_data.1.len()) }
  fn size_alloc(cre: &Krate) -> usize { (size_of::<AstId<SpecAny>>() * cre.extra_data.0.allocated_len()) + (size_of::<NodeKind>() * cre.extra_data.1.allocated_len()) }
}
