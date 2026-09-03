use std::num::NonZeroU32;

use lasso::{Key, Rodeo, Spur};


pub struct StrInterner {
  rodeo: Rodeo,
}

impl StrInterner {
  #[inline]
  pub fn new() -> Self {
    Self {
      rodeo: Rodeo::default(),
    }
  }


  #[inline]
  pub fn sid(&mut self, s: &str) -> NonZeroU32 {
    let spur = self.rodeo.get_or_intern(s);
    NonZeroU32::new((spur.into_usize() +1) as u32).unwrap()
  }

  #[inline]
  pub fn str(&self, sid: NonZeroU32) -> &str {
    let spur = Spur::try_from_usize(sid.get() as usize -1).unwrap();
    self.rodeo.resolve(&spur)
  }

}
