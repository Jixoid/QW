/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::{FmtString, Span};


#[derive(Clone, Debug)]
pub struct Label {
  pub(crate) span: Span,
  pub(crate) message: Option<String>,
}


impl Label {
  pub fn new(span: impl Into<Span>, msg: impl Into<FmtString>) -> Self {
    Label { span: span.into(), message: Some(msg.into().msg) }
  }

  pub fn new_pos(span: impl Into<Span>) -> Self {
    Label { span: span.into(), message: None }
  }
}
