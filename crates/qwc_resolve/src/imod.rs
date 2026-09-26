use qwc_string_interner::Sid;
use crate::ExportMap;


#[derive(Clone, Copy)]
pub struct Imod<'a> {
  pub name: Sid,
  pub expmap: &'a ExportMap,
}

impl<'a> Imod<'a> {
  pub fn new(name: Sid, expmap: &'a ExportMap) -> Self {
    Self { name, expmap }
  }
}
