/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::Add;

use backtrace::Backtrace;
use thin_vec::ThinVec;

use crate::{CodedString, FmtMsg, FmtString, ICE, Label, Suggestion};


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
    let bt = Backtrace::new();
    let str = format!("\n{}", ICE::bt(bt));

    Message { level: Level::Fatal, message: msg.into(), labels: ThinVec::new(), notes: ThinVec::from([str]), suggestions: ThinVec::new() }
  }

  pub fn fatal_pos(msg: impl Into<CodedString>, label: Label) -> Self {
    let bt = Backtrace::new();
    let str = format!("\n{}", ICE::bt(bt));

    Message { level: Level::Fatal, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::from([str]), suggestions: ThinVec::new() }
  }

  pub fn error(msg: impl Into<CodedString>, label: Label) -> Self {
    Message { level: Level::Error, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }
  
  pub fn error_nopos(msg: impl Into<CodedString>) -> Self {
    Message { level: Level::Error, message: msg.into(), labels: ThinVec::new(), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn warn(msg: impl Into<CodedString>, label: Label) -> Self {
    Message { level: Level::Warn, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }

  pub fn hint(msg: impl Into<CodedString>, label: Label) -> Self {
    Message { level: Level::Hint, message: msg.into(), labels: ThinVec::from([label]), notes: ThinVec::new(), suggestions: ThinVec::new() }
  }
}


impl Add<Label> for Message {
  type Output = Self;

  fn add(mut self, rhs: Label) -> Self::Output {
    self.labels.push(rhs);
    self
  }
}

impl Add<Suggestion> for Message {
  type Output = Self;

  fn add(mut self, rhs: Suggestion) -> Self::Output {
    self.suggestions.push(rhs);
    self
  }
}

impl Add<FmtMsg> for Message {
  type Output = Self;

  fn add(mut self, rhs: FmtMsg) -> Self::Output {
    self.notes.push(rhs.msg.to_string());
    self
  }
}

impl Add<FmtString> for Message {
  type Output = Self;

  fn add(mut self, rhs: FmtString) -> Self::Output {
    self.notes.push(rhs.msg);
    self
  }
}


impl Add<Option<Label>> for Message {
  type Output = Self;

  fn add(mut self, rhs: Option<Label>) -> Self::Output {
    if let Some(rhs) = rhs { self.labels.push(rhs) };
    self
  }
}

impl Add<Option<Suggestion>> for Message {
  type Output = Self;
  
  fn add(mut self, rhs: Option<Suggestion>) -> Self::Output {
    if let Some(rhs) = rhs { self.suggestions.push(rhs) };
    self
  }
}

impl Add<Option<FmtMsg>> for Message {
  type Output = Self;
  
  fn add(mut self, rhs: Option<FmtMsg>) -> Self::Output {
    if let Some(rhs) = rhs { self.notes.push(rhs.msg.to_string()) };
    self
  }
}

impl Add<Option<FmtString>> for Message {
  type Output = Self;
  
  fn add(mut self, rhs: Option<FmtString>) -> Self::Output {
    if let Some(rhs) = rhs { self.notes.push(rhs.msg) };
    self
  }
}
