use qwc_ast::{Attribute, Visibility};
use qwc_diagnostic::{Label, Message, msg::*};
use qwc_lexer::{WK, Word};

use crate::{Ctx, Fail, AttrParser, ctx};


pub struct MetaParser;

impl MetaParser {

  pub fn pmr_global(ctx: &mut Ctx) -> Result<(),  Message> { ctx!(ctx => cre, sin, far, lex, sum);
    let mut level: isize = 0;

    loop {
      let t = match lex.get() {
        Ok(t) => t,
        Err(..) => return Ok(()),
      };

      if t.kind() == WK::BraceL { level += 1; continue; }
      if t.kind() == WK::BraceR {
        level -= 1;

        if level <= 0 { return Ok(()); }
      }
    }
  }


  pub fn read_start(ctx: &mut Ctx, defvis: &mut Visibility) -> Result<(Visibility, Option<Vec<Attribute>>), Message> { ctx!(ctx => cre, sin, far, lex, sum);
    loop {
      let attr = if lex.peek()?.kind() == WK::BangAttr { AttrParser::read_attr(ctx!(cre, sin, far, lex, sum))? } else { None };

      let vis = match lex.peek()?.kind() {
        WK::Pub   => Some((lex.get()?, Visibility::Public)),
        WK::Priv  => Some((lex.get()?, Visibility::Private)),
        WK::Prot  => Some((lex.get()?, Visibility::Protected)),
        WK::Crate => Some((lex.get()?, Visibility::Crate)),
        WK::Super => Some((lex.get()?, Visibility::Super)),
      
        _ => None
      };

      if let Some((w, vis)) = vis {
        if lex.peek()?.kind() == WK::Colon {
          if let Some(..) = attr {
            return Err(Message::error(VISIBILITY_AFTER_ATTRIBUTE, Label::new_pos(w)))
          }

          lex.bump()?;
          *defvis = vis;
          continue;
        } 
        break Ok((vis, attr));
      }
      break Ok((*defvis, attr));
    }
  }

}


impl<'a> WordCheck for Word {

  fn expect_kind(self, k1: WK) -> Result<Self, Message> {
    match self.kind() == k1 {
      true  => Ok(self),
      false => Err(Message::error(EXPECTED_BUT_FOUND
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      ))
    }
  }

  
  fn panic_kind(self, k1: WK) -> Fail<Message> {
    match self.kind() == k1 {
      true  => panic!(),
      false => Err(Message::error(EXPECTED_BUT_FOUND
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      ))
    }
  }

  fn panic_kind2(self, k1: WK, k2: WK) -> Fail<Message> {
    match self.kind() == k1 || self.kind() == k2 {
      true  => panic!(),
      false => Err(Message::error(EXPECTED_BUT_FOUND2
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", k2),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      ))
    }
  }
  
  fn panic_kind3(self, k1: WK, k2: WK, k3: WK) -> Fail<Message> {
    match self.kind() == k1 || self.kind() == k2 || self.kind() == k3 {
      true  => panic!(),
      false => Err(Message::error(EXPECTED_BUT_FOUND3
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", k2),
          &format!("{:?}", k3),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      ))
    }
  }
  
  fn panic_kind4(self, k1: WK, k2: WK, k3: WK, k4: WK) -> Fail<Message> {
    match self.kind() == k1 || self.kind() == k2 || self.kind() == k3 || self.kind() == k4 {
      true  => panic!(),
      false => Err(Message::error(EXPECTED_BUT_FOUND4
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", k2),
          &format!("{:?}", k3),
          &format!("{:?}", k4),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      ))
    }
  }

}

pub trait WordCheck {
  fn expect_kind(self, k1: WK) -> Result<Word, Message>;

  fn panic_kind(self, k1: WK) -> Fail<Message>;
  fn panic_kind2(self, k1: WK, k2: WK) -> Fail<Message>;
  fn panic_kind3(self, k1: WK, k2: WK, k3: WK) -> Fail<Message>;
  fn panic_kind4(self, k1: WK, k2: WK, k3: WK, k4: WK) -> Fail<Message>;
}
