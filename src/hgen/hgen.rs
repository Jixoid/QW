use crate::{ast, hir};


pub struct HGen {}

impl<'a> HGen {

  pub fn lower(_ast: &ast::Crate) -> hir::Crate<'a> {
    hir::Crate::new()
  }

}
