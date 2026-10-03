use core::slice;

use qwc_arena::Arena;

use crate::{AnyId, AnyRng, Block, BlokId, Inst, Rng, SymbId, Symbol, Type, TypeId, Value, ValueId, id::{InstId, MirId, MirKind, NodeKind, SpecAny}};


pub struct Krate {
  // Arena
  list_type: Arena<Type>,
  list_symb: Arena<Symbol>,
  list_blok: Arena<Block>,
  list_inst: Arena<Inst>,
  list_valu: Arena<Value>,

  extra_data: (Arena<MirId<SpecAny>>, Arena<NodeKind>),

  // Symbol String
  symbols: Vec<String>,
}


impl Krate {

  pub fn new() -> Self {
    Self{
      list_type: Arena::new(),
      list_symb: Arena::new(),
      list_blok: Arena::new(),
      list_inst: Arena::new(),
      list_valu: Arena::new(),
      
      extra_data: (Arena::new(), Arena::new()),

      symbols: vec![],
    }
  }


  // Arena
  pub fn push<T: PushApi>(&mut self, obj: T) -> T::Id { T::push(self, obj) }
  pub fn get<I: GetApi>(&self, id: I) -> &I::Node { I::get(self, id) }
  pub fn get_mut<I: GetApi>(&mut self, id: I) -> &mut I::Node { I::get_mut(self, id) }
  pub fn iter<T: PushApi>(&self) -> impl Iterator<Item = &T> { T::iter(self) }
  pub fn iter_mut<T: PushApi>(&mut self) -> impl Iterator<Item = &mut T> { T::iter_mut(self) }


  // Extra
  pub fn extra<T: MirKind>(&mut self, vec: &[MirId<T>]) -> Rng<T> {
    let (ids, kds) = &mut self.extra_data;

    let vec = unsafe { slice::from_raw_parts(vec.as_ptr() as *const MirId<SpecAny>, vec.len()) };

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

  pub fn extra_get<T: MirKind>(&self, rng: Rng<T>) -> impl Iterator<Item = MirId<T>> {
    let (ids, kinds) = &self.extra_data;
    let range = rng.range();

    std::iter::zip(
      ids.range(range.clone()),
      kinds.range(range),
    )
    .map(|(&id, &kind)| MirId::<T>::new_from((id, kind)))
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
    Self::size_used::<Type>(&self) + Self::size_used::<Symbol>(&self) + Self::size_used::<Block>(&self) + Self::size_used::<Inst>(&self) + Self::size_used::<Value>(&self) + Self::size_used::<AnyId>(&self)
  }

  pub fn size_all_alloc(&self) -> usize {
    Self::size_alloc::<Type>(&self) + Self::size_alloc::<Symbol>(&self) + Self::size_alloc::<Block>(&self) + Self::size_alloc::<Inst>(&self) + Self::size_alloc::<Value>(&self) + Self::size_alloc::<AnyId>(&self)
  }

}



// push & get
pub trait PushApi: 'static {
  type Id;
  
  fn push(krate: &mut Krate, obj: Self) -> Self::Id;
  fn iter<'a>(krate: &'a Krate) -> impl Iterator<Item = &'a Self>;
  fn iter_mut<'a>(krate: &'a mut Krate) -> impl Iterator<Item = &'a mut Self>;
}

pub trait GetApi {
  type Node;

  fn get<'a>(krate: &'a Krate, id: Self) -> &'a Self::Node;
  fn get_mut<'a>(krate: &'a mut Krate, id: Self) -> &'a mut Self::Node;
}

impl PushApi for Type {
  type Id = TypeId;
  
  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_type.push(obj)).unwrap()) }
  fn iter<'a>(krate: &'a Krate) -> impl Iterator<Item = &'a Self> { krate.list_type.iter() }
  fn iter_mut<'a>(krate: &'a mut Krate) -> impl Iterator<Item = &'a mut Self> { krate.list_type.iter_mut() }
}

impl GetApi for TypeId {
  type Node = Type;

  fn get<'a>(krate: &'a Krate, id: Self) -> &'a Self::Node { &krate.list_type[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self) -> &'a mut Self::Node { &mut krate.list_type[id.idx() as usize] }
}

impl PushApi for Symbol {
  type Id = SymbId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_symb.push(obj)).unwrap()) }
  fn iter<'a>(krate: &'a Krate) -> impl Iterator<Item = &'a Self> { krate.list_symb.iter() }
  fn iter_mut<'a>(krate: &'a mut Krate) -> impl Iterator<Item = &'a mut Self> { krate.list_symb.iter_mut() }
}

impl GetApi for SymbId {
  type Node = Symbol;

  fn get<'a>(krate: &'a Krate, id: Self) -> &'a Self::Node { &krate.list_symb[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self) -> &'a mut Self::Node { &mut krate.list_symb[id.idx() as usize] }
}

impl PushApi for Block {
  type Id = BlokId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_blok.push(obj)).unwrap()) }
  fn iter<'a>(krate: &'a Krate) -> impl Iterator<Item = &'a Self> { krate.list_blok.iter() }
  fn iter_mut<'a>(krate: &'a mut Krate) -> impl Iterator<Item = &'a mut Self> { krate.list_blok.iter_mut() }
}

impl GetApi for BlokId {
  type Node = Block;

  fn get<'a>(krate: &'a Krate, id: Self) -> &'a Self::Node { &krate.list_blok[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self) -> &'a mut Self::Node { &mut krate.list_blok[id.idx() as usize] }
}

impl PushApi for Inst {
  type Id = InstId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_inst.push(obj)).unwrap()) }
  fn iter<'a>(krate: &'a Krate) -> impl Iterator<Item = &'a Self> { krate.list_inst.iter() }
  fn iter_mut<'a>(krate: &'a mut Krate) -> impl Iterator<Item = &'a mut Self> { krate.list_inst.iter_mut() }
}

impl GetApi for InstId {
  type Node = Inst;

  fn get<'a>(krate: &'a Krate, id: Self) -> &'a Self::Node { &krate.list_inst[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self) -> &'a mut Self::Node { &mut krate.list_inst[id.idx() as usize] }
}

impl PushApi for Value {
  type Id = ValueId;

  fn push(krate: &mut Krate, obj: Self) -> Self::Id { Self::Id::new(u32::try_from(krate.list_valu.push(obj)).unwrap()) }
  fn iter<'a>(krate: &'a Krate) -> impl Iterator<Item = &'a Self> { krate.list_valu.iter() }
  fn iter_mut<'a>(krate: &'a mut Krate) -> impl Iterator<Item = &'a mut Self> { krate.list_valu.iter_mut() }
}

impl GetApi for ValueId {
  type Node = Value;

  fn get<'a>(krate: &'a Krate, id: Self) -> &'a Self::Node { &krate.list_valu[id.idx() as usize] }
  fn get_mut<'a>(krate: &'a mut Krate, id: Self) -> &'a mut Self::Node { &mut krate.list_valu[id.idx() as usize] }
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

impl SizeApi for Block {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_blok.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_blok.allocated_len() }
}

impl SizeApi for Inst {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_inst.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_inst.allocated_len() }
}

impl SizeApi for Value {
  fn size_used(cre: &Krate) -> usize { size_of::<Self>() * cre.list_valu.len() }
  fn size_alloc(cre: &Krate) -> usize { size_of::<Self>() * cre.list_valu.allocated_len() }
}

impl SizeApi for AnyId {
  fn size_used(cre: &Krate) -> usize { (size_of::<MirId<SpecAny>>() * cre.extra_data.0.len()) + (size_of::<NodeKind>() * cre.extra_data.1.len()) }
  fn size_alloc(cre: &Krate) -> usize { (size_of::<MirId<SpecAny>>() * cre.extra_data.0.allocated_len()) + (size_of::<NodeKind>() * cre.extra_data.1.allocated_len()) }
}
