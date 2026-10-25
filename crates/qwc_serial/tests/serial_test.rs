use std::any::{Any, TypeId};
use qwc_serial::{DecodeCtx, Decoder, EncodeCtx, EncodeErr, Encoder, MapEncoder, SerialMapEncode};
use qwc_serial_macro::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct SimpleStruct {
  a: u32,
  b: String,
  c: Option<i64>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
enum SimpleEnum {
  Unit,
  Single(u32),
  Named { x: i32, y: String },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
#[serial(map)]
struct TestMapped(u32);

struct TestMapEncoder {
  items: Vec<TestMapped>,
}

impl SerialMapEncode<TestMapped> for TestMapEncoder {
  fn map_encode(&mut self, val: &TestMapped) -> Result<u32, EncodeErr> {
    if let Some(pos) = self.items.iter().position(|it| it == val) {
      Ok(pos as u32)
    } else {
      let idx = self.items.len() as u32;
      self.items.push(*val);
      Ok(idx)
    }
  }
}

impl MapEncoder for TestMapEncoder {
  fn map_encode_any(&mut self, type_id: TypeId, val: &dyn Any) -> Result<Option<u32>, EncodeErr> {
    if type_id == TypeId::of::<TestMapped>() {
      let v = val.downcast_ref::<TestMapped>().unwrap();
      Ok(Some(self.map_encode(v)?))
    } else {
      Ok(None)
    }
  }
}

#[test]
fn test_struct_roundtrip() {
  let s = SimpleStruct {
    a: 42,
    b: "hello world".to_string(),
    c: Some(-12345),
  };
  let mut buf = Vec::new();
  let mut enc = EncodeCtx::new(&mut buf);
  s.encode(&mut enc).unwrap();

  let mut slice = &buf[..];
  let mut dec = DecodeCtx::new(&mut slice);
  let decoded = SimpleStruct::decode(&mut dec).unwrap();
  assert_eq!(s, decoded);
}

#[test]
fn test_enum_roundtrip() {
  let items = vec![
    SimpleEnum::Unit,
    SimpleEnum::Single(999),
    SimpleEnum::Named { x: -5, y: "test".to_string() },
  ];
  let mut buf = Vec::new();
  let mut enc = EncodeCtx::new(&mut buf);
  items.encode(&mut enc).unwrap();

  let mut slice = &buf[..];
  let mut dec = DecodeCtx::new(&mut slice);
  let decoded = Vec::<SimpleEnum>::decode(&mut dec).unwrap();
  assert_eq!(items, decoded);
}

#[test]
fn test_mapped_roundtrip() {
  let mut map_enc = TestMapEncoder { items: Vec::new() };
  let original = vec![TestMapped(100), TestMapped(200), TestMapped(100), TestMapped(300)];

  let mut buf = Vec::new();
  {
    let mut enc = EncodeCtx::new_with_map(&mut buf, &mut map_enc);
    original.encode(&mut enc).unwrap();
  }

  // Deserializing with local slice decoder
  let mut slice = &buf[..];
  let mut dec = DecodeCtx::new_with_map(&mut slice, &map_enc.items);
  let decoded = Vec::<TestMapped>::decode(&mut dec).unwrap();
  assert_eq!(original, decoded);
}
