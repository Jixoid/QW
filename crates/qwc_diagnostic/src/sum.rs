use core::fmt;
use std::ops;
use owo_colors::OwoColorize;

use crate::{Level, Message};


#[derive(Default, Clone)]
pub struct Summary {
  fatal: u32, error: u32, warn: u32, hint: u32,
  msgs: Vec<Message>,
}


impl Summary {

  pub fn new() -> Self {
    Self{fatal: 0, error: 0, warn: 0, hint: 0, msgs: vec![]}
  }

  pub fn add(&mut self, msg: Message) {
    match msg.level {
      Level::Fatal => self.fatal += 1,
      Level::Error => self.error += 1,
      Level::Warn => self.warn += 1,
      Level::Hint => self.hint += 1,
    };

    self.msgs.push(msg);
  }

  pub fn is_empty(&self) -> bool { return (self.fatal + self.error + self.warn + self.hint) == 0; }
  
  pub fn msgs(&self) -> &Vec<Message> { return &self.msgs; }
  
  pub fn sumerr(&self) -> u32 { return self.fatal + self.error; }
}

impl IntoIterator for Summary {
  type Item = Message;
  type IntoIter = std::vec::IntoIter<Message>;

  fn into_iter(self) -> Self::IntoIter { self.msgs.into_iter() }
}

impl<'a> IntoIterator for &'a Summary {
  type Item = &'a Message;
  type IntoIter = std::slice::Iter<'a, Message>;

  fn into_iter(self) -> Self::IntoIter { self.msgs.iter() }
}

impl<'a> fmt::Display for Summary {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if !self.is_empty() {
      let mut parts = Vec::new();
      if self.fatal != 0 { parts.push(format!("{} {}", self.fatal, "fatal")); }
      if self.error != 0 { parts.push(format!("{} {}", self.error, "error")); }
      if self.warn != 0  { parts.push(format!("{} {}", self.warn, "warn")); }
      if self.hint != 0  { parts.push(format!("{} {}", self.hint, "hint")); }
      let str = parts.join(", ");
      
      if self.sumerr() == 0 {
        write!(f, "{}{} {} {}", "hint".yellow().bold(), ":".bright_black(), "compilation was completed with these", str)?
      } else {
        write!(f, "{}{} {} {}", "fatal".red().bold(), ":".bright_black(), "could not compile due to", str)?
      }
    }

    Ok(())
  }
}

impl ops::AddAssign for Summary {
  fn add_assign(&mut self, rhs: Self) {
    self.fatal += rhs.fatal;
    self.error += rhs.error;
    self.warn  += rhs.warn;
    self.hint  += rhs.hint;

    self.msgs.extend(rhs.msgs);
  }
}
