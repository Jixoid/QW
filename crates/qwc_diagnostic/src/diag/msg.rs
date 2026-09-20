use thin_vec::ThinVec;

use crate::{CodedString, FmtMsg, FmtString, Label, Suggestion};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level { Fatal, Error, Warn, Hint }


#[derive(Clone, Debug)]
pub struct Message {
  pub(crate) level: Level,
  pub(crate) message: CodedString,
  pub(crate) labels: ThinVec<Label>,
  pub(crate) notes: ThinVec<String>,
  pub(crate) suggestions: ThinVec<Suggestion>,
}

impl Message {
  pub fn fatal(msg: impl Into<CodedString>) -> Self {
    Message { level: Level::Fatal, message: msg.into(), labels: ThinVec::new(), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn error(msg: impl Into<CodedString>, label: Label) -> Self {
    Message { level: Level::Error, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn warn(msg: impl Into<CodedString>, label: Label) -> Self {
    Message { level: Level::Warn, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn hint(msg: impl Into<CodedString>, label: Label) -> Self {
    Message { level: Level::Hint, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }


  pub fn add(mut self, it: impl MsgAddApi) -> Self {
    match it.msg_get() {
      MsgAdd::Note(it) => self.notes.push(it),
      MsgAdd::Label(it) => self.labels.push(it),
      MsgAdd::Sugg(it) => self.suggestions.push(it),
    }
    self
  }

  
  pub fn add_if(mut self, it: Option<impl MsgAddApi>) -> Self {
    if let Some(it) = it {
      match it.msg_get() {
        MsgAdd::Note(it) => self.notes.push(it),
        MsgAdd::Label(it) => self.labels.push(it),
        MsgAdd::Sugg(it) => self.suggestions.push(it),
      }
    }
    self
  }
}



pub enum MsgAdd {
  Note(String),
  Label(Label),
  Sugg(Suggestion),
}

pub trait MsgAddApi {
  fn msg_get(self) -> MsgAdd;
}


impl MsgAddApi for FmtMsg {
  fn msg_get(self) -> MsgAdd { MsgAdd::Note(self.msg.to_string()) }
}

impl MsgAddApi for FmtString {
  fn msg_get(self) -> MsgAdd { MsgAdd::Note(self.msg) }
}

impl MsgAddApi for Label {
  fn msg_get(self) -> MsgAdd { MsgAdd::Label(self) }
}

impl MsgAddApi for Suggestion {
  fn msg_get(self) -> MsgAdd { MsgAdd::Sugg(self) }
}
