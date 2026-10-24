/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use qwc_ast::{Attribute, Visibility};
use qwc_diagnostic::{Label, Message, Status::{self, Fail, Success}, Summary, msg::*, ret_err};
use qwc_lexer::{WK, Word};

use crate::{Ctx, AttrParser, ctx};


pub struct MetaParser;

impl MetaParser {

  pub fn pmr_global(ctx: &mut Ctx) -> Status<()> { ctx!(ctx => cre, sin, far, lex, sum, side);
    let mut level: isize = 0;

    loop {
      let t = match lex.get(sum) {
        Success(t) => t,
        Fail => return Success(()),
      };

      if t.kind() == WK::BraceL { level += 1; continue; }
      if t.kind() == WK::BraceR {
        level -= 1;

        if level <= 0 { return Success(()); }
      }
    }
  }


  pub fn read_start(ctx: &mut Ctx, defvis: &mut Visibility) -> Status<(Visibility, Option<Vec<Attribute>>)> { ctx!(ctx => cre, sin, far, lex, sum, side);
    loop {
      let attr = if lex.peek(sum)?.kind() == WK::BangAttr { AttrParser::read_attr(ctx!(cre, sin, far, lex, sum, side))? } else { None };

      let vis = match lex.peek(sum)?.kind() {
        WK::Pub   => Some((lex.get(sum)?, Visibility::Public)),
        WK::Priv  => Some((lex.get(sum)?, Visibility::Private)),
        WK::Prot  => Some((lex.get(sum)?, Visibility::Protected)),
        WK::Crate => Some((lex.get(sum)?, Visibility::Crate)),
        WK::Super => Some((lex.get(sum)?, Visibility::Super)),
      
        _ => None
      };

      if let Some((w, vis)) = vis {
        if lex.peek(sum)?.kind() == WK::Colon {
          if attr.is_some() {
            ret_err!(sum, Message::error(VISIBILITY_AFTER_ATTRIBUTE, Label::new_pos(w)));
          }

          lex.bump(sum)?;
          *defvis = vis;
          continue;
        } 
        break Success((vis, attr));
      }
      break Success((*defvis, attr));
    }
  }

}


impl WordCheck for Word {

  fn expect_kind(self, sum: &mut Summary, k1: WK) -> Status<Self> {
    match self.kind() == k1 {
      true  => Success(self),
      false => ret_err!(sum, Message::error(EXPECTED_BUT_FOUND
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      )),
    }
  }

  
  fn panic_kind(self, sum: &mut Summary, k1: WK) -> Status<!> {
    match self.kind() == k1 {
      true  => unreachable!(),
      false => ret_err!(sum, Message::error(EXPECTED_BUT_FOUND
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      )),
    }
  }

  fn panic_kind2(self, sum: &mut Summary, k1: WK, k2: WK) -> Status<!> {
    match self.kind() == k1 || self.kind() == k2 {
      true  => unreachable!(),
      false => ret_err!(sum, Message::error(EXPECTED_BUT_FOUND2
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", k2),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      )),
    }
  }
  
  fn panic_kind3(self, sum: &mut Summary, k1: WK, k2: WK, k3: WK) -> Status<!> {
    match self.kind() == k1 || self.kind() == k2 || self.kind() == k3 {
      true  => unreachable!(),
      false => ret_err!(sum, Message::error(EXPECTED_BUT_FOUND3
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", k2),
          &format!("{:?}", k3),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      )),
    }
  }
  
  fn panic_kind4(self, sum: &mut Summary, k1: WK, k2: WK, k3: WK, k4: WK) -> Status<!> {
    match self.kind() == k1 || self.kind() == k2 || self.kind() == k3 || self.kind() == k4 {
      true  => unreachable!(),
      false => ret_err!(sum, Message::error(EXPECTED_BUT_FOUND4
        .args(&[
          &format!("{:?}", k1),
          &format!("{:?}", k2),
          &format!("{:?}", k3),
          &format!("{:?}", k4),
          &format!("{:?}", self.kind()),
        ]),
        Label::new_pos(self)
      )),
    }
  }

}

pub trait WordCheck {
  fn expect_kind(self, sum: &mut Summary, k1: WK) -> Status<Word>;

  fn panic_kind(self, sum: &mut Summary, k1: WK) -> Status<!>;
  fn panic_kind2(self, sum: &mut Summary, k1: WK, k2: WK) -> Status<!>;
  fn panic_kind3(self, sum: &mut Summary, k1: WK, k2: WK, k3: WK) -> Status<!>;
  fn panic_kind4(self, sum: &mut Summary, k1: WK, k2: WK, k3: WK, k4: WK) -> Status<!>;
}
