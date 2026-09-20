use std::fmt;

use thin_vec::ThinVec;

use crate::{CodedString, Label};


#[derive(Clone, Copy, Debug)]
pub enum Applicability {
  MachineApplicable,
  MaybeIncorrect,
}

impl fmt::Display for Applicability {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Applicability::MaybeIncorrect => write!(f, "maybe incorrect"),
      Applicability::MachineApplicable => write!(f, "applicable"),
    }
  }
}


#[derive(Clone, Debug)]
pub struct Suggestion {
  pub(crate) labels: ThinVec<Label>,
  pub(crate) message: CodedString,
  pub(crate) replacement: String,
  pub(crate) applicability: Applicability,
}

impl Suggestion {
  pub fn new(msg: impl Into<CodedString>, applicability: Applicability, replacement: String, label: Label) -> Self {
    Self { message: msg.into(), labels: ThinVec::from([label]), replacement, applicability }
  }

  
  pub fn add(mut self, it: impl SuggAddApi) -> Self {
    match it.sugg_get() {
      SuggAdd::Label(it) => self.labels.push(it),
    }
    self
  }
}



pub enum SuggAdd {
  Label(Label),
}

pub trait SuggAddApi {
  fn sugg_get(self) -> SuggAdd;
}


impl SuggAddApi for Label {
  fn sugg_get(self) -> SuggAdd { SuggAdd::Label(self) }
}
