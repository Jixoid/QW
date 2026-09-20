use crate::{FmtString, Span};


#[derive(Clone, Debug)]
pub struct Label {
  pub(crate) span: Span,
  pub(crate) message: Option<String>,
}


impl Label {
  pub fn new(span: impl Into<Span>, msg: impl Into<FmtString>) -> Self {
    Label { span: span.into(), message: Some(msg.into().msg) }
  }

  pub fn new_pos(span: impl Into<Span>) -> Self {
    Label { span: span.into(), message: None }
  }
}
