use core::fmt;
use owo_colors::OwoColorize;

use crate::lexer::Word;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MsgKind { Fatal, Error, Warn, Hint, Note }


#[derive(Clone)]
pub struct Message<'a> {
  kind: MsgKind,
  pos: Word<'a>,
  msg: String,
  pars: Vec<String>,
  notes: Vec<Message<'a>>,
}

impl<'a> Message<'a> {
  pub fn new(kind: MsgKind, pos: Word<'a>, msg: String, pars: Vec<String>) -> Message<'a> { Message {kind, pos, msg, pars, notes: Vec::new()} }

  pub fn fatal(pos: Word<'a>, msg: String, pars: Vec<String>) -> Message<'a> { Self::new(MsgKind::Fatal, pos, msg, pars) }
  pub fn error(pos: Word<'a>, msg: String, pars: Vec<String>) -> Message<'a> { Self::new(MsgKind::Error, pos, msg, pars) }
  pub fn warn(pos: Word<'a>, msg: String, pars: Vec<String>) -> Message<'a>  { Self::new(MsgKind::Warn, pos, msg, pars) }
  pub fn hint(pos: Word<'a>, msg: String, pars: Vec<String>) -> Message<'a>  { Self::new(MsgKind::Hint, pos, msg, pars) }
  pub fn note(pos: Word<'a>, msg: String, pars: Vec<String>) -> Message<'a>  { Self::new(MsgKind::Note, pos, msg, pars) }

  pub fn add_note(&mut self, m: Message<'a>) { self.notes.push(m); }
}

impl<'a> fmt::Display for Message<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // File
    let (hr1, _hr2) = self.pos.interval();
    let mut fpath = self.pos.mol.fpath.clone();
    
    if let crate::control::module::ModuleKind::RTL = self.pos.mol.kind {
      if let Some(src_pos) = fpath.find("/src/") {
        fpath = format!("[rtl]/{}", &fpath[src_pos + 5..]);
      }
    }

    write!(f, "{}{}{}{}{}{} ", 
      fpath.blue().bold(),
      ":".bright_black(),
      hr1.line,
      ":".bright_black(),
      hr1.column,
      ":".bright_black()
    )?;


    // Message
    match self.kind {
      MsgKind::Fatal => write!(f, "{}", "fatal".bright_red().bold())?,
      MsgKind::Error => write!(f, "{}", "error".bright_red().bold())?,
      MsgKind::Warn  => write!(f, "{}", "warn".bright_yellow().bold())?,
      MsgKind::Hint  => write!(f, "{}", "hint".bright_green().bold())?,
      MsgKind::Note  => write!(f, "{}", "note".bright_green().bold())?,
    };
    
    let formatted_msg = format_from_vector(&self.msg, &self.pars);
    writeln!(f, "{}{}{}", ":".bright_black(), " ", formatted_msg)?;


    let file = self.pos.mol.mmap.as_str();
    let ctx_word = self.pos;
    let (hr1, hr2) = ctx_word.interval();

    let off = ctx_word.off;
    let size = ctx_word.size;
    let end_off = off + size;

    let beg = match file[..off].rfind('\n') {
      Some(idx) => idx + 1,
      None => 0,
    };

    let end = match file[end_off..].find('\n') {
      Some(idx) => end_off + idx,
      None => file.len(),
    };

    let max_line = std::cmp::max(hr1.line, hr2.line);
    let padd = max_line.to_string().len();
    let mut line = hr1.line;

    let mut line_buf = String::new();
    let mut under_buf = String::new();
    let mut has_underline = false;

    let bytes = file.as_bytes();
    for i in beg..=end {
      let c = if i < file.len() { bytes[i] as char } else { '\n' };

      if c != '\n' && i != end {
        line_buf.push(c);

        if i == off {
          under_buf.push('^');
          has_underline = true;
        } else if i > off && i < end_off {
          under_buf.push('~');
          has_underline = true;
        } else {
          under_buf.push(if c == '\t' { '\t' } else { ' ' });
        }
      }

      if c == '\n' || i == end {
        if i == end && line_buf.is_empty() && !has_underline && c == '\n' {
          break;
        }

        let l_str = line.to_string();
        let spaces = " ".repeat(padd.saturating_sub(l_str.len()) + 1);
        writeln!(f, "  {}{}{}{}", spaces, line, " | ".bright_black(), line_buf)?;

        if has_underline {
          let u_spaces = " ".repeat(padd + 1);
          writeln!(f, "  {}{}{}", u_spaces, " | ".bright_black(), under_buf.bright_green())?;
        }

        line += 1;
        line_buf.clear();
        under_buf.clear();
        has_underline = false;
      }
    }

    for n in &self.notes {
      write!(f, "{}", n)?;
    }

    Ok(())
  }
}


fn format_from_vector(fmt_str: &str, v: &[String]) -> String {
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



#[derive(Clone)]
pub struct Summary<'a> {
  fatal: u32, error: u32, warn: u32, hint: u32, note: u32,
  msgs: Vec<Message<'a>>,
}

impl<'a> Summary<'a> {

  pub fn new() -> Summary<'a> { Summary {fatal: 0, error: 0, warn: 0, hint: 0, note: 0, msgs: Vec::new()} }

  pub fn add(&mut self, m: Message<'a>) {
    match m.kind {
      MsgKind::Fatal => self.fatal += 1,
      MsgKind::Error => self.error += 1,
      MsgKind::Warn => self.warn += 1,
      MsgKind::Hint => self.hint += 1,
      MsgKind::Note => self.note += 1,
    };

    self.msgs.push(m);
  }

  pub fn msgs(&self) -> &Vec<Message<'a>> { return &self.msgs; }

  pub fn sumall(&self) -> u32 { return self.fatal + self.error + self.warn + self.hint + self.note; }
  pub fn sumerr(&self) -> u32 { return self.fatal + self.error; }
}

impl<'a> fmt::Display for Summary<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.sumall() == 0 { write!(f, "") } else {
      let mut str = format!("{}{} ", "summary".bright_yellow().bold(), ":".bright_black());

      let mut parts = Vec::new();
      if self.fatal != 0 { parts.push(format!("{}{} {}", "fatal".bright_red().bold(), ":".bright_black(), self.fatal)); }
      if self.error != 0 { parts.push(format!("{}{} {}", "error".bright_red().bold(), ":".bright_black(), self.error)); }
      if self.warn != 0 { parts.push(format!("{}{} {}", "warn".bright_yellow().bold(), ":".bright_black(), self.warn)); }
      if self.hint != 0 { parts.push(format!("{}{} {}", "hint".bright_green().bold(), ":".bright_black(), self.hint)); }
      if self.note != 0 { parts.push(format!("{}{} {}", "note".bright_green().bold(), ":".bright_black(), self.note)); }
      str += &parts.join(", ");
      
      return write!(f, "{}", str)
    }
  }
}
