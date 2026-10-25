/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{any::{Any, TypeId}, error::Error, fmt, io::{Read, Write}, marker::PhantomData, num::NonZero};

use byteorder::{LittleEndian, ReadBytesExt};
use rustc_hash::FxHashMap;


// Map Traits
pub trait SerialMapEncode<T> {
  fn map_encode(&mut self, val: &T) -> Result<u32, EncodeErr>;
  fn as_map_encoder<'a>(&'a mut self) -> MappedEncode<'a, T, Self> where Self: Sized {
    MappedEncode(self, PhantomData)
  }
}

pub trait SerialMapDecode<T> {
  fn map_decode(&self, id: u32) -> Result<T, DecodeErr>;
  fn as_map_decoder<'a>(&'a self) -> MappedDecode<'a, T, Self> where Self: Sized {
    MappedDecode(self, PhantomData)
  }
}

pub trait MapEncoder {
  fn map_encode_any(&mut self, type_id: TypeId, val: &dyn Any) -> Result<Option<u32>, EncodeErr>;
}

pub trait MapDecoder {
  fn map_decode_any(&self, type_id: TypeId, id: u32, out: &mut dyn Any) -> Result<bool, DecodeErr>;
}

pub struct MappedEncode<'a, T, M: SerialMapEncode<T>>(pub &'a mut M, pub PhantomData<T>);

impl<'a, T: 'static, M: SerialMapEncode<T>> MapEncoder for MappedEncode<'a, T, M> {
  fn map_encode_any(&mut self, type_id: TypeId, val: &dyn Any) -> Result<Option<u32>, EncodeErr> {
    if type_id == TypeId::of::<T>() {
      let typed_val = val.downcast_ref::<T>().ok_or(EncodeErr::ImpossibleData)?;
      let id = self.0.map_encode(typed_val)?;
      Ok(Some(id))
    } else {
      Ok(None)
    }
  }
}

pub struct MappedDecode<'a, T, M: SerialMapDecode<T>>(pub &'a M, pub PhantomData<T>);

impl<'a, T: 'static, M: SerialMapDecode<T>> MapDecoder for MappedDecode<'a, T, M> {
  fn map_decode_any(&self, type_id: TypeId, id: u32, out: &mut dyn Any) -> Result<bool, DecodeErr> {
    if type_id == TypeId::of::<T>() {

      let slot = out.downcast_mut::<Option<T>>().ok_or(DecodeErr::ImpossibleData)?;
      let val = self.0.map_decode(id)?;
      *slot = Some(val);
      Ok(true)
    } else {
      Ok(false)
    }
  }
}

impl<T: Copy + 'static> SerialMapDecode<T> for [T] {
  fn map_decode(&self, id: u32) -> Result<T, DecodeErr> {
    self.get(id as usize).copied().ok_or(DecodeErr::ImpossibleData)
  }
}

impl<T: Copy + 'static> SerialMapDecode<T> for Vec<T> {
  fn map_decode(&self, id: u32) -> Result<T, DecodeErr> {
    self.as_slice().map_decode(id)
  }
}

impl<T: Copy + 'static> MapDecoder for [T] {
  fn map_decode_any(&self, type_id: TypeId, id: u32, out: &mut dyn Any) -> Result<bool, DecodeErr> {
    if type_id == TypeId::of::<T>() {
      let slot = out.downcast_mut::<Option<T>>().ok_or(DecodeErr::ImpossibleData)?;
      let val = self.get(id as usize).copied().ok_or(DecodeErr::ImpossibleData)?;
      *slot = Some(val);
      Ok(true)
    } else {
      Ok(false)
    }
  }
}

impl<T: Copy + 'static> MapDecoder for Vec<T> {
  fn map_decode_any(&self, type_id: TypeId, id: u32, out: &mut dyn Any) -> Result<bool, DecodeErr> {
    self.as_slice().map_decode_any(type_id, id, out)
  }
}

pub struct SliceDecoder<'a, T: Copy + 'static>(pub &'a [T]);

impl<'a, T: Copy + 'static> MapDecoder for SliceDecoder<'a, T> {
  fn map_decode_any(&self, type_id: TypeId, id: u32, out: &mut dyn Any) -> Result<bool, DecodeErr> {
    if type_id == TypeId::of::<T>() {
      let slot = out.downcast_mut::<Option<T>>().ok_or(DecodeErr::ImpossibleData)?;
      let val = self.0.get(id as usize).copied().ok_or(DecodeErr::ImpossibleData)?;
      *slot = Some(val);
      Ok(true)
    } else {
      Ok(false)
    }
  }
}




// Encode
pub struct EncodeCtx<'a> {
  pub file: &'a mut dyn Write,
  pub written: usize,
  pub map: Option<&'a mut dyn MapEncoder>,
}

impl<'a> EncodeCtx<'a> {
  pub fn new(file: &'a mut dyn Write) -> Self {
    Self { file, written: 0, map: None }
  }

  pub fn new_with_map(file: &'a mut dyn Write, map: &'a mut dyn MapEncoder) -> Self {
    Self { file, written: 0, map: Some(map) }
  }

  pub fn new_with_offset(file: &'a mut dyn Write, written: usize) -> Self {
    Self { file, written, map: None }
  }

  #[inline]
  pub fn write_all(&mut self, buf: &[u8]) -> Result<(), EncodeErr> {
    self.file.write_all(buf)?;
    self.written += buf.len();
    Ok(())
  }

  #[inline]
  pub fn written(&self) -> usize {
    self.written
  }

  pub fn map_encode<T: 'static>(&mut self, val: &T) -> Result<u32, EncodeErr> {
    if let Some(ref mut map) = self.map {
      if let Some(id) = map.map_encode_any(TypeId::of::<T>(), val)? {
        return Ok(id);
      }
    }
    Err(EncodeErr::MissingMap)
  }
}


#[derive(Debug)]
pub enum EncodeErr {
  DataWasNotWritten,
  MissingMap,
  ImpossibleData,
}

impl fmt::Display for EncodeErr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::DataWasNotWritten => write!(f, "data was not written"),
      Self::MissingMap => write!(f, "missing map for mapped type"),
      Self::ImpossibleData => write!(f, "impossible data"),
    }
  }
}

impl Error for EncodeErr {}

impl From<std::io::Error> for EncodeErr {
  fn from(_: std::io::Error) -> Self {
    Self::DataWasNotWritten
  }
}

impl From<EncodeErr> for String {
  fn from(val: EncodeErr) -> Self {
    format!("{}", val)
  }
}

pub trait Encoder {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr>;
}


// Decode
pub struct DecodeCtx<'a> {
  pub file: &'a mut dyn Read,
  pub map: Option<&'a dyn MapDecoder>,
}

impl<'a> DecodeCtx<'a> {
  pub fn new(file: &'a mut dyn Read) -> Self {
    Self { file, map: None }
  }

  pub fn new_with_map(file: &'a mut dyn Read, map: &'a dyn MapDecoder) -> Self {
    Self { file, map: Some(map) }
  }

  pub fn map_decode<T: 'static>(&mut self, id: u32) -> Result<T, DecodeErr> {
    if let Some(map) = self.map {
      let mut out: Option<T> = None;
      if map.map_decode_any(TypeId::of::<T>(), id, &mut out)? {
        return out.ok_or(DecodeErr::ImpossibleData);
      }
    }
    Err(DecodeErr::MissingMap)
  }
}

#[derive(Debug)]
pub enum DecodeErr {
  DataWasNotReaded,
  UnknownTag,
  ImpossibleData,
  MissingMap,
}

impl fmt::Display for DecodeErr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::DataWasNotReaded => write!(f, "data was not readed"),
      Self::UnknownTag => write!(f, "unknown tag"),
      Self::ImpossibleData => write!(f, "impossible data"),
      Self::MissingMap => write!(f, "missing map for mapped type"),
    }
  }
}

impl Error for DecodeErr {}

impl From<std::io::Error> for DecodeErr {
  fn from(_: std::io::Error) -> Self {
    Self::DataWasNotReaded
  }
}

impl From<DecodeErr> for String {
  fn from(val: DecodeErr) -> Self {
    format!("{}", val)
  }
}

pub trait Decoder: Sized {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr>;
}


// Primitive Encoders
impl Encoder for u128 { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for u64  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for u32  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for u16  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for u8   { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for usize { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { (*self as u64).encode(ctx) } }

impl Encoder for i128 { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for i64  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for i32  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for i16  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for i8   { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.to_le_bytes()) } }
impl Encoder for isize { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { (*self as i64).encode(ctx) } }

impl Encoder for NonZero<u128> { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<u64>  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<u32>  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<u16>  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<u8>   { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<usize> { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { NonZero::<u64>::new(self.get() as u64).unwrap().encode(ctx) } }

impl Encoder for NonZero<i128> { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<i64>  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<i32>  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<i16>  { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<i8>   { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { ctx.write_all(&self.get().to_le_bytes()) } }
impl Encoder for NonZero<isize> { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { NonZero::<i64>::new(self.get() as i64).unwrap().encode(ctx) } }


impl Encoder for bool { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { if *self { 1u8 } else { 0u8 }.encode(ctx) }}

impl Encoder for char { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { (*self as u32).encode(ctx) } }

impl Encoder for str {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    (self.len() as u32).encode(ctx)?;
    ctx.write_all(self.as_bytes())
  }
}

impl Encoder for String { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { self.as_str().encode(ctx) } }

impl<T> Encoder for PhantomData<T> { fn encode(&self, _: &mut EncodeCtx) -> Result<(), EncodeErr> { Ok(()) } }

impl<T: Encoder> Encoder for Option<T> {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    if let Some(val) = self {
      true.encode(ctx)?;
      val.encode(ctx)?;
    } else {
      false.encode(ctx)?;
    }
    Ok(())
  }
}

impl<T: Encoder> Encoder for Box<T> { fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> { self.as_ref().encode(ctx) } }

impl<T: Encoder> Encoder for Vec<T> {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    (self.len() as u32).encode(ctx)?;
    for it in self {
      it.encode(ctx)?;
    }
    Ok(())
  }
}

impl<const N: usize> Encoder for [u8; N] {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    ctx.write_all(self)
  }
}


impl<A: Encoder, B: Encoder> Encoder for (A, B) {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    self.0.encode(ctx)?;
    self.1.encode(ctx)?;
    Ok(())
  }
}

impl<K: Encoder, V: Encoder> Encoder for FxHashMap<K, V> {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    (self.len() as u32).encode(ctx)?;
    for (k, v) in self {
      k.encode(ctx)?;
      v.encode(ctx)?;
    }
    Ok(())
  }
}


// Primitive Decoders
impl Decoder for u128 { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u128::<LittleEndian>()?; Ok(me) } }
impl Decoder for u64  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u64::<LittleEndian>()?; Ok(me) } }
impl Decoder for u32  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u32::<LittleEndian>()?; Ok(me) } }
impl Decoder for u16  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u16::<LittleEndian>()?; Ok(me) } }
impl Decoder for u8   { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u8()?; Ok(me) } }
impl Decoder for usize { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let val = u64::decode(ctx)?; Ok(val as usize) } }

impl Decoder for i128 { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i128::<LittleEndian>()?; Ok(me) } }
impl Decoder for i64  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i64::<LittleEndian>()?; Ok(me) } }
impl Decoder for i32  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i32::<LittleEndian>()?; Ok(me) } }
impl Decoder for i16  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i16::<LittleEndian>()?; Ok(me) } }
impl Decoder for i8   { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i8()?; Ok(me) } }
impl Decoder for isize { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let val = i64::decode(ctx)?; Ok(val as isize) } }

impl Decoder for NonZero<u128> { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u128::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<u64>  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u64::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<u32>  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u32::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<u16>  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u16::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<u8>   { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_u8()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<usize> { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let val = NonZero::<u64>::decode(ctx)?; NonZero::new(val.get() as usize).ok_or(DecodeErr::ImpossibleData) } }

impl Decoder for NonZero<i128> { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i128::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<i64>  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i64::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<i32>  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i32::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<i16>  { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i16::<LittleEndian>()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<i8>   { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let me = ctx.file.read_i8()?; NonZero::new(me).ok_or(DecodeErr::ImpossibleData) } }
impl Decoder for NonZero<isize> { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let val = NonZero::<i64>::decode(ctx)?; NonZero::new(val.get() as isize).ok_or(DecodeErr::ImpossibleData) } }

impl Decoder for bool { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { let val = u8::decode(ctx)?; Ok(val != 0) } }

impl Decoder for char {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let val = u32::decode(ctx)?;
    char::from_u32(val).ok_or(DecodeErr::ImpossibleData)
  }
}

impl Decoder for String {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let len = u32::decode(ctx)? as usize;
    let mut buf = vec![0u8; len];
    ctx.file.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|_| DecodeErr::ImpossibleData)
  }
}

impl<T> Decoder for PhantomData<T> { fn decode(_: &mut DecodeCtx) -> Result<Self, DecodeErr> { Ok(PhantomData) } }

impl<T: Decoder> Decoder for Option<T> {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let is_some = bool::decode(ctx)?;
    if is_some {
      Ok(Some(T::decode(ctx)?))
    } else {
      Ok(None)
    }
  }
}

impl<T: Decoder> Decoder for Box<T> { fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> { Ok(Box::new(T::decode(ctx)?)) } }

impl<T: Decoder> Decoder for Vec<T> {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let len = u32::decode(ctx)? as usize;
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
      vec.push(T::decode(ctx)?);
    }
    Ok(vec)
  }
}

impl<const N: usize> Decoder for [u8; N] {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let mut buf = [0u8; N];
    ctx.file.read_exact(&mut buf)?;
    Ok(buf)
  }
}

impl<A: Decoder, B: Decoder> Decoder for (A, B) {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let a = A::decode(ctx)?;
    let b = B::decode(ctx)?;
    Ok((a, b))
  }
}

impl<K: Decoder + Eq + std::hash::Hash, V: Decoder> Decoder for FxHashMap<K, V> {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    let len = u32::decode(ctx)? as usize;
    let mut map = FxHashMap::with_capacity_and_hasher(len, Default::default());
    for _ in 0..len {
      let k = K::decode(ctx)?;
      let v = V::decode(ctx)?;
      map.insert(k, v);
    }
    Ok(map)
  }
}

// Float codecs
impl Encoder for rustc_apfloat::ieee::Half {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    use rustc_apfloat::Float;
    (self.to_bits() as u16).encode(ctx)
  }
}

impl Encoder for rustc_apfloat::ieee::Single {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    use rustc_apfloat::Float;
    (self.to_bits() as u32).encode(ctx)
  }
}

impl Encoder for rustc_apfloat::ieee::Double {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    use rustc_apfloat::Float;
    (self.to_bits() as u64).encode(ctx)
  }
}

impl Encoder for rustc_apfloat::ieee::Quad {
  fn encode(&self, ctx: &mut EncodeCtx) -> Result<(), EncodeErr> {
    use rustc_apfloat::Float;
    self.to_bits().encode(ctx)
  }
}

impl Decoder for rustc_apfloat::ieee::Half {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    use rustc_apfloat::Float;
    let val = u16::decode(ctx)?;
    Ok(Self::from_bits(val as u128))
  }
}

impl Decoder for rustc_apfloat::ieee::Single {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    use rustc_apfloat::Float;
    let val = u32::decode(ctx)?;
    Ok(Self::from_bits(val as u128))
  }
}

impl Decoder for rustc_apfloat::ieee::Double {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    use rustc_apfloat::Float;
    let val = u64::decode(ctx)?;
    Ok(Self::from_bits(val as u128))
  }
}

impl Decoder for rustc_apfloat::ieee::Quad {
  fn decode(ctx: &mut DecodeCtx) -> Result<Self, DecodeErr> {
    use rustc_apfloat::Float;
    let val = u128::decode(ctx)?;
    Ok(Self::from_bits(val))
  }
}
