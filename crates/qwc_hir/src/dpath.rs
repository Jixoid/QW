use std::num::NonZeroU32;

use thin_vec::ThinVec;


pub struct DPath(pub ThinVec<NonZeroU32>);

impl DPath {
  
  pub fn new(vec: Vec<NonZeroU32>) -> Self {
    Self(ThinVec::from(vec))
  }

}
