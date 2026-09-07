use qwc_diagnostic::Summary;

use crate::Krate;


pub trait Visitor {
  type Type;

  fn visit(cre: &Krate) -> Result<Self::Type, Summary>;
}
