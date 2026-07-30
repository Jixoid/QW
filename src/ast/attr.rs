use crate::lexer::Word;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute<'a> {
  pub key: Word<'a>,
  pub val: Option<Word<'a>>,
}
