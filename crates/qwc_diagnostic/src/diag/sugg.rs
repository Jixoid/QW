/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{fmt, ops::Add};

use thin_vec::ThinVec;

use crate::{CodedString, Label};


#[derive(Clone, Copy, Debug)]
pub enum Applicability {
  MachineApplicable,
  MaybeIncorrect,
}

impl fmt::Display for Applicability {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Applicability::MaybeIncorrect => write!(f, "maybe incorrect"),
      Applicability::MachineApplicable => write!(f, "applicable"),
    }
  }
}


#[derive(Clone, Debug)]
pub struct Suggestion {
  pub(crate) labels: ThinVec<Label>,
  pub(crate) message: CodedString,
  pub(crate) replacement: String,
  pub(crate) applicability: Applicability,
}

impl Suggestion {
  pub fn new(msg: impl Into<CodedString>, applicability: Applicability, replacement: String, label: Label) -> Self {
    Self { message: msg.into(), labels: ThinVec::from([label]), replacement, applicability }
  }
}

impl Add<Label> for Suggestion {
  type Output = Self;

  fn add(mut self, rhs: Label) -> Self::Output {
    self.labels.push(rhs);
    self
  }
}
