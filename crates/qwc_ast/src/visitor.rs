use qwc_arena::Files;
use qwc_diagnostic::Summary;
use qwc_string_interner::StrInterner;

use crate::Krate;


pub trait Visitor {
  type Type;

  fn visit(cre: &Krate, sin: &StrInterner, far: &Files) -> Result<Self::Type, Summary>;
}
