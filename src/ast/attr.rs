use crate::lexer::Word;

#[derive(Debug, Clone, Copy)]
pub struct Attribute<'a> {
  pub key: Word<'a>,
  pub val: Option<Word<'a>>,
}
