use thin_vec::ThinVec;

use crate::Span;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level { Fatal, Error, Warn, Hint }


#[derive(Clone, Debug)]
pub struct DiagCode ( pub(crate) u16 );


#[derive(Clone, Debug)]
pub struct CodedMsg {
  pub(crate) code: Option<DiagCode>,
  pub(crate) msg: &'static str
}

impl CodedMsg {
  pub const fn new(code: u16, msg: &'static str) -> Self {
    Self{code: Some(DiagCode(code)), msg}
  }

  pub const fn new_str(msg: &'static str) -> Self {
    Self{code: None, msg}
  }
}

impl Into<String> for CodedMsg {
  fn into(self) -> String {
    if let Some(code) = self.code {
      format!("[{}]{}", code.0, self.msg)
    } else {
      self.msg.to_string()
    }
  }
}


#[derive(Clone, Debug)]
pub struct Label {
  pub(crate) span: Span,
  pub(crate) message: Option<String>,
}

impl Label {
  
  pub fn new(span: impl Into<Span>, msg: &'static str) -> Self {
    Label { span: span.into(), message: Some(msg.into()) }
  }

  pub fn new_pos(span: impl Into<Span>) -> Self {
    Label { span: span.into(), message: None }
  }

  pub fn new_args(span: impl Into<Span>, msg: &'static str, args: &[&str]) -> Self {
    Label { span: span.into(), message: Some(format_from_vector(msg, args)) }
  }

}

fn format_from_vector(str: &str, args: &[&str]) -> String {
  let mut res = String::new();
  let mut parts = str.split("{}");
  
  if let Some(first) = parts.next() {
    res.push_str(first);
  }
  
  let mut v_iter = args.iter();
  for part in parts {
    if let Some(arg) = v_iter.next() {
      res.push_str(arg);
    }
    res.push_str(part);
  }
  
  res
}


#[derive(Clone, Debug)]
pub struct Suggestion {
  span: Span,
  replacement: String,
  message: String,
  applicability: Applicability,
}

#[derive(Clone, Copy, Debug)]
pub enum Applicability {
  MachineApplicable,
  MaybeIncorrect,
  HasPlaceholders,
}

#[derive(Clone, Debug)]
pub struct Message {
  pub(crate) level: Level,
  pub(crate) message: CodedMsg,
  pub(crate) labels: ThinVec<Label>,
  pub(crate) notes: ThinVec<&'static str>,
  suggestions: ThinVec<Suggestion>,
}

impl Message {

  pub fn fatal(msg: impl Into<CodedMsg>) -> Self {
    Message { level: Level::Fatal, message: msg.into(), labels: ThinVec::new(), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn error(msg: impl Into<CodedMsg>, label: Label) -> Self {
    Message { level: Level::Error, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn warn(msg: impl Into<CodedMsg>, label: Label) -> Self {
    Message { level: Level::Warn, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn hint(msg: impl Into<CodedMsg>, label: Label) -> Self {
    Message { level: Level::Hint, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }


  pub fn add_note(mut self, msg: &'static str) -> Self {
    self.notes.push(msg.into());
    self
  }

  pub fn add_label(mut self, label: Label) -> Self {
    self.labels.push(label);
    self
  }

}
