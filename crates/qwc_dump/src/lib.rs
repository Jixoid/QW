use std::fmt;
use owo_colors::{OwoColorize, Style};

#[derive(Clone, Copy)]
pub struct Styled<T> {
  pub val: T,
  pub style: Style,
}

impl<T: fmt::Display> fmt::Display for Styled<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.val.style(self.style))
  }
}

pub fn kw<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().blue().bold() }
}

pub fn name<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().yellow().bold() }
}

pub fn punct<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().bright_black() }
}

pub fn op<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().bright_black() }
}

pub fn tmpval<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().magenta().bold() }
}

pub fn ty<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().cyan().bold() }
}

pub fn lit_num<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().yellow() }
}

pub fn lit_bool(b: bool) -> Styled<&'static str> {
  Styled {
    val: if b { "true" } else { "false" },
    style: Style::new().yellow(),
  }
}

pub fn lit_str<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().green() }
}

pub fn attr<T>(val: T) -> Styled<T> {
  Styled { val, style: Style::new().bright_magenta() }
}

pub fn write_indent(f: &mut fmt::Formatter, indent: usize) -> fmt::Result {
  for _ in 0..indent {
    write!(f, "  ")?;
  }
  Ok(())
}

pub fn write_delimited<I, F>(f: &mut fmt::Formatter, iter: I, sep: &str, mut dump_fn: F) -> fmt::Result
where
  I: IntoIterator,
  F: FnMut(&mut fmt::Formatter, I::Item) -> fmt::Result,
{
  let mut first = true;
  for item in iter {
    if !first {
      write!(f, "{}", sep)?;
    }
    first = false;
    dump_fn(f, item)?;
  }
  Ok(())
}
