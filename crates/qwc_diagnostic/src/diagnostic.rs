use core::fmt;
use std::{ops, path::Path};
use owo_colors::OwoColorize;
use qwc_arena::Files;

use crate::Span;


#[derive(Copy, Clone, PartialEq, Eq)]
enum MsgKind { Fatal, Error, Warn, Hint, Note }

#[derive(Clone)]
pub struct Message {
  pos: Span,
  kind: MsgKind,
  msg: String,
  notes: Vec<Message>,
}

impl Message {
  fn new(kind: MsgKind, pos: Span, msg: &str, pars: &[&str]) -> Self { Message {kind, pos, msg: format_from_vector(msg, pars), notes: vec![]} }

  pub fn fatal(pos: impl Into<Span>, msg: &str, pars: &[&str]) -> Self { Self::new(MsgKind::Fatal, pos.into(), msg, pars) }
  pub fn error(pos: impl Into<Span>, msg: &str, pars: &[&str]) -> Self { Self::new(MsgKind::Error, pos.into(), msg, pars) }
  pub fn warn(pos: impl Into<Span>, msg: &str, pars: &[&str]) -> Self  { Self::new(MsgKind::Warn, pos.into(), msg, pars) }
  pub fn hint(pos: impl Into<Span>, msg: &str, pars: &[&str]) -> Self  { Self::new(MsgKind::Hint, pos.into(), msg, pars) }
  pub fn note(pos: impl Into<Span>, msg: &str, pars: &[&str]) -> Self  { Self::new(MsgKind::Note, pos.into(), msg, pars) }
  
  pub fn add_note(&mut self, m: Self) { self.notes.push(m); }

  pub fn display<'a>(&'a self, far: &'a Files) -> MessageDisplay<'a> { MessageDisplay(self, far) }
}


pub struct MessageDisplay<'a> (&'a Message, &'a Files);


impl<'a> fmt::Display for MessageDisplay<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let msg = self.0;
    let far = self.1;

    /* Error Desc */ {
      match msg.kind {
        MsgKind::Fatal => write!(f, "{}", "fatal".bright_red().bold())?,
        MsgKind::Error => write!(f, "{}", "error".bright_red().bold())?,
        MsgKind::Warn  => write!(f, "{}", "warn".bright_yellow().bold())?,
        MsgKind::Hint  => write!(f, "{}", "hint".bright_green().bold())?,
        MsgKind::Note  => write!(f, "{}", "note".bright_green().bold())?,
      };
      
      writeln!(f, "{}{}{}", ":".bright_black(), " ", msg.msg)?;
    }


    let file = str::from_utf8(&far.get(msg.pos.fid).map()[..]).unwrap();

    let hr = msg.pos.interval(far);

    let rng = msg.pos.range();

    let beg = file[..rng.start].rfind('\n').map(|x| x +1).unwrap_or(0);
    let end = file[rng.end..].find('\n').map(|x| rng.end +x).unwrap_or(file.len());

    let padd = std::cmp::max(hr.start[0], hr.end[0]).to_string().len();
    

    /* Error File : Pos */ {
      let raw_fpath = far.get(msg.pos.fid).fpath();
      let path = Path::new(raw_fpath);
      let path = path.strip_prefix(".").unwrap_or(path);
      let display_path = if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd).unwrap_or(path)
      } else {
        path
      };
      let display_path = display_path.strip_prefix(".").unwrap_or(display_path);
      let space = " ".repeat(padd+1);
      
      writeln!(f, "{}{} {}{}{}{}{}",
        space,
        "-->".blue().bold(),

        display_path.display().to_string().blue().bold(),
        ":".bright_black(),
        hr.start[0],
        ":".bright_black(),
        hr.start[1],
      )?;

      writeln!(f, "{} {}", space, "|".bright_black())?;
    }


    /* File Show */ {
      let mut line = hr.start[0].get();
      let mut line_buf = String::new();
      let mut under_buf = String::new();
      let mut has_underline = false;
      
      let bytes = file.as_bytes();
      for i in beg..=end {
        let c = if i < file.len() { bytes[i] as char } else { '\n' };

        if c != '\n' && i != end {
          line_buf.push(c);

          if i >= (rng.start as usize) && i < rng.end {
            under_buf.push('^');
            has_underline = true;
          } else {
            under_buf.push(if c == '\t' { '\t' } else { ' ' });
          }
        }

        if c == '\n' || i == end {
          if i == end && line_buf.is_empty() && !has_underline && c == '\n' { break }

          let spaces = " ".repeat(padd.saturating_sub(line.to_string().len()) +1);
          writeln!(f, "{}{} {} {}", spaces, line, "|".bright_black(), line_buf)?;

          if has_underline {
            let u_spaces = " ".repeat(padd + 1);
            writeln!(f, "{} {} {}", u_spaces, "|".bright_black(), under_buf.yellow().bold())?;
          }

          line += 1;
          line_buf.clear();
          under_buf.clear();
          has_underline = false;
        }
      }
    }

    for n in &msg.notes {
      write!(f, "{}", n.display(far))?;
    }

    Ok(())
  }
}


fn format_from_vector(fmt_str: &str, v: &[&str]) -> String {
  let mut res = String::new();
  let mut parts = fmt_str.split("{}");
  
  if let Some(first) = parts.next() {
    res.push_str(first);
  }
  
  let mut v_iter = v.iter();
  for part in parts {
    if let Some(arg) = v_iter.next() {
      res.push_str(arg);
    }
    res.push_str(part);
  }
  
  res
}



#[derive(Default, Clone)]
pub struct Summary {
  fatal: u32, error: u32, warn: u32, hint: u32, note: u32,
  msgs: Vec<Message>,
}

impl Summary {

  pub fn new() -> Self {
    Self{fatal: 0, error: 0, warn: 0, hint: 0, note: 0, msgs: vec![]}
  }

  pub fn add(&mut self, m: Message) {
    match m.kind {
      MsgKind::Fatal => self.fatal += 1,
      MsgKind::Error => self.error += 1,
      MsgKind::Warn => self.warn += 1,
      MsgKind::Hint => self.hint += 1,
      MsgKind::Note => self.note += 1,
    };

    self.msgs.push(m);
  }

  pub fn is_empty(&self) -> bool { return (self.fatal + self.error + self.warn + self.hint + self.note) == 0; }
  
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
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if !self.is_empty() {
      let mut str = format!("{}{} ", "summary".bright_yellow().bold(), ":".bright_black());

      let mut parts = Vec::new();
      if self.fatal != 0 { parts.push(format!("{}{} {}", "fatal".bright_red().bold(), ":".bright_black(), self.fatal)); }
      if self.error != 0 { parts.push(format!("{}{} {}", "error".bright_red().bold(), ":".bright_black(), self.error)); }
      if self.warn != 0 { parts.push(format!("{}{} {}", "warn".bright_yellow().bold(), ":".bright_black(), self.warn)); }
      if self.hint != 0 { parts.push(format!("{}{} {}", "hint".bright_green().bold(), ":".bright_black(), self.hint)); }
      if self.note != 0 { parts.push(format!("{}{} {}", "note".bright_green().bold(), ":".bright_black(), self.note)); }
      str += &parts.join(", ");
      
      write!(f, "{}", str)?
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
    self.note  += rhs.note;

    self.msgs.extend(rhs.msgs);
  }
}
