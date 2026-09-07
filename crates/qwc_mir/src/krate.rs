use core::slice;

use qwc_arena::Arena;

use crate::{AnyId, Block, BlokId, SymbId, Symbol, Type, TypeId, Value, id::{MirId, MirKind, NodeKind, SpecAny, ValuId}, ty_interner::TypeInterner};


pub struct Krate {
  // Arena
  pub(super) list_type: Arena<Type>,
  list_symb: Arena<Symbol>,
  list_valu: Arena<Value>,
  list_blok: Arena<Block>,

  extra_data: (Arena<MirId<SpecAny>>, Arena<NodeKind>),

  // Symbol String
  symbols: Vec<String>,

  // Type Interning
  pub(super) tyin: Option<TypeInterner>,
}


impl Krate {

  pub fn new() -> Self {
    let mut ret = Self{
      list_type: Arena::new(),
      list_symb: Arena::new(),
      list_valu: Arena::new(),
      list_blok: Arena::new(),
      
      extra_data: (Arena::new(), Arena::new()),

      symbols: vec![],

      tyin: None,
    };

    ret.tyin = Some(TypeInterner::new(&mut ret));

    ret
  }


  // Arena
  pub fn push<T: ArenaNode>(&mut self, obj: T) -> T::Id { T::push(self, obj) }
  pub fn get<T: ArenaNode>(&self, id: T::Id) -> &T { T::get(self, id) }
  pub fn get_mut<T: ArenaNode>(&mut self, id: T::Id) -> &mut T { T::get_mut(self, id) }


  // Extra
  pub fn extra<T: MirKind>(&mut self, vec: &[MirId<T>]) -> Rng {
    let (ids, kds) = &mut self.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const MirId<SpecAny>, vec.len()) };

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

  pub fn extra_get(&self, rng: Rng) -> impl Iterator<Item = (MirId<SpecAny>, NodeKind)> {
    let (ids, kinds) = &self.extra_data;
    let range = (rng.0 as usize)..(rng.1 as usize);

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| (id, kind))
  }


  // Symbol
  pub fn sym(&mut self, str: &str) -> u32 {
    self.symbols.push(str.to_string());
    self.symbols.len() as u32 -1
  }

  pub fn sym_str(&self, id: u32) -> &str {
    &self.symbols[id as usize]
  }

  pub fn symbols_len(&self) -> usize {
    self.list_symb.len()
  }

  pub fn types_len(&self) -> usize {
    self.list_type.len()
  }

  pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
    self.list_symb.iter()
  }

  pub fn types(&self) -> impl Iterator<Item = &Type> {
    self.list_type.iter()
  }


  // Size
  pub fn size_used<T: SizeApi>(&self) -> usize { T::size_used(&self) }
  pub fn size_alloc<T: SizeApi>(&self) -> usize { T::size_alloc(&self) }
  
  pub fn size_all_used(&self) -> usize {
    Self::size_used::<Type>(&self) + Self::size_used::<Symbol>(&self) + Self::size_used::<Value>(&self) + Self::size_used::<Block>(&self) + Self::size_used::<AnyId>(&self)
  }

  pub fn size_all_alloc(&self) -> usize {
    Self::size_alloc::<Type>(&self) + Self::size_alloc::<Symbol>(&self) + Self::size_alloc::<Value>(&self) + Self::size_alloc::<Block>(&self) + Self::size_alloc::<AnyId>(&self)
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

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_type.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_type[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_type[id.idx() as usize] }
}

impl ArenaNode for Symbol {
  type Id = SymbId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_symb.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_symb[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_symb[id.idx() as usize] }
}

impl ArenaNode for Value {
  type Id = ValuId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_valu.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_valu[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_valu[id.idx() as usize] }
}

impl ArenaNode for Block {
  type Id = BlokId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_blok.push(obj)).unwrap()) }
  fn get<'a>(krate: &'a Krate, id: Self::Id) -> &'a Self { &krate.list_blok[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self::Id) -> &'a mut Self { &mut krate.list_blok[id.idx() as usize] }
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

impl SizeApi for Symbol {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_symb.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_symb.allocated_len() }
}

impl SizeApi for Value {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_valu.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_valu.allocated_len() }
}

impl SizeApi for Block {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_blok.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_blok.allocated_len() }
}

impl SizeApi for AnyId {
  fn size_used(cre: &Krate) -> usize { (size_of::<MirId<SpecAny>>() * cre.extra_data.0.len()) + (size_of::<NodeKind>() * cre.extra_data.1.len()) }
  fn size_alloc(cre: &Krate) -> usize { (size_of::<MirId<SpecAny>>() * cre.extra_data.0.allocated_len()) + (size_of::<NodeKind>() * cre.extra_data.1.allocated_len()) }
}
