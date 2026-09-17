use std::num::NonZeroU32;

use lasso::{Key, Rodeo, Spur};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Sid (NonZeroU32);

impl From<NonZeroU32> for Sid {
  fn from(value: NonZeroU32) -> Self { Self(value) }
}


pub struct StrInterner {
  rodeo: Rodeo,

  sid_sys: Sid,
  sid_types: Sid,
  
  sid_bool: Sid,
  
  sid_usize: Sid,
  sid_isize: Sid,

  sid_import: Sid,
  sid_export: Sid,
}

impl StrInterner {
  
  pub fn new() -> Self {
    let mut rodeo = Rodeo::default();

    Self {
      sid_sys:   Sid::from(rodeo.get_or_intern_static("sys").into_inner()),
      sid_types: Sid::from(rodeo.get_or_intern_static("types").into_inner()),
      
      sid_bool: Sid::from(rodeo.get_or_intern_static("bool").into_inner()),
      
      sid_usize: Sid::from(rodeo.get_or_intern_static("usize").into_inner()),
      sid_isize: Sid::from(rodeo.get_or_intern_static("isize").into_inner()),

      sid_import: Sid::from(rodeo.get_or_intern_static("import").into_inner()),
      sid_export: Sid::from(rodeo.get_or_intern_static("export").into_inner()),
      
      rodeo,
    }
  }


  #[inline]
  pub fn sid(&mut self, s: &str) -> Sid {
    let spur = self.rodeo.get_or_intern(s);
    Sid::from(spur.into_inner())
  }

  #[inline]
  pub fn str(&self, sid: Sid) -> &str {
    let spur = Spur::try_from_usize(sid.0.get() as usize -1).unwrap();
    self.rodeo.resolve(&spur)
  }


  pub fn sid_sys(&self) -> Sid   { self.sid_sys }
  pub fn sid_types(&self) -> Sid { self.sid_types }

  pub fn sid_bool(&self) -> Sid { self.sid_bool }
  
  pub fn sid_usize(&self) -> Sid { self.sid_usize }
  pub fn sid_isize(&self) -> Sid { self.sid_isize }

  pub fn sid_import(&self) -> Sid { self.sid_import }  
  pub fn sid_export(&self) -> Sid { self.sid_export }  
}
