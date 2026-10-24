/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{cmp::Ordering, fmt, str::FromStr};

use itertools::Itertools;


#[derive(PartialEq, Eq)]
pub struct Version {
  pub(crate) major: u32,
  pub(crate) minor: u32,
  pub(crate) patch: u32,
  
  pub(crate) branch: Option<String>,
}

impl Version {
  pub fn ver(&self) -> (u32, u32, u32) {
    (self.major, self.minor, self.patch)
  }

  pub fn branch(&self) -> Option<&str> {
    match &self.branch {
      Some(v) => Some(v.as_str()),
      None => None,
    }
  }
}

impl PartialOrd for Version {
  fn partial_cmp(&self, it: &Self) -> Option<Ordering> {
    match (self.major, self.minor, self.patch).cmp(&(it.major, it.minor, it.patch)) {
      Ordering::Equal => {
        match (&self.branch, &it.branch) {
          (None, None) => Some(Ordering::Equal),
          (None, Some(_)) => Some(Ordering::Greater),
          (Some(_), None) => Some(Ordering::Less),
          (Some(_), Some(_)) => None,
        }
      }
      
      ord @_ => Some(ord),
    }
  }
}

impl FromStr for Version {
  type Err = ParseVersionError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    let input = input.trim().trim_start_matches('v');
    if input.is_empty() {
      return Err(ParseVersionError::EmptyInput);
    }

    // '-' işaretinden bölerek Pre-release kısmını ayır
    let (core_version, branch) = match input.split_once('-') {
      Some((left, pre)) if !pre.is_empty() => (left, Some(pre.to_string())),
      Some(_) => return Err(ParseVersionError::InvalidFormat(input.to_string())),
      None => (input, None),
    };

    // MAJOR.MINOR.PATCH basamaklarını '.' ile ayır ve u64'e çevir
    let parts = core_version.split('.').collect_vec();
    if parts.len() != 3 {
      return Err(ParseVersionError::InvalidFormat(core_version.to_string()));
    }

    let parse_num = |s: &str| -> Result<u32, ParseVersionError> {
      s.parse::<u32>().map_err(|_| ParseVersionError::InvalidNumber(s.to_string()))
    };

    Ok(Version {
      major: parse_num(parts[0])?,
      minor: parse_num(parts[1])?,
      patch: parse_num(parts[2])?,
      branch,
    })
  }
}


#[derive(Copy, Clone)]
pub enum Operator {
  Caret,
  Tilde,
  Exact,
  GreaterEq,
  Less,
  Wildcard,
}


pub struct VerReq {
  pub(crate) op: Operator,
  pub(crate) ver: Option<Version>,
}

impl VerReq {
  pub fn op(&self) -> Operator {
    self.op
  }

  pub fn ver(&self) -> Option<&Version> {
    self.ver.as_ref()
  }

  pub fn matches(&self, candidate: &Version) -> bool {
    let Some(base) = &self.ver else {
      return candidate.branch.is_none();
    };

    if candidate.branch.is_some() && base.branch.is_none() {
      return false;
    }

    match self.op {
      Operator::Wildcard => true,
      Operator::Exact => candidate == base,
      Operator::GreaterEq => candidate >= base,
      Operator::Less => candidate < base,
      Operator::Tilde => candidate >= base && candidate.major == base.major && candidate.minor == base.minor,
      Operator::Caret => {
        if candidate < base {
          return false;
        }

        if base.major > 0 {
          candidate.major == base.major
        }
        else if base.minor > 0 {
          candidate.major == 0 && candidate.minor == base.minor
        }
        else {
          candidate.major == 0 && candidate.minor == 0 && candidate.patch == base.patch
        }
      }
    }
  }
}

impl FromStr for VerReq {
  type Err = ParseVersionError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    let s = input.trim();
    if s.is_empty() {
      return Err(ParseVersionError::EmptyInput);
    }

    if s == "*" {
      return Ok(VerReq {
        op: Operator::Wildcard,
        ver: None,
      });
    }

    // Başlangıçtaki operatörü tespit et ve geri kalan sürüm metnini ayır
    let (op, rest) = if let Some(r) = s.strip_prefix(">=") {
      (Operator::GreaterEq, r)
    } else if let Some(r) = s.strip_prefix('^') {
      (Operator::Caret, r)
    } else if let Some(r) = s.strip_prefix('~') {
      (Operator::Tilde, r)
    } else if let Some(r) = s.strip_prefix('=') {
      (Operator::Exact, r)
    } else if let Some(r) = s.strip_prefix('<') {
      (Operator::Less, r)
    } else {
      (Operator::Exact, s)
    };

    let version = Version::from_str(rest)?;
    Ok(VerReq {
      op,
      ver: Some(version),
    })
  }
}



pub enum ParseVersionError {
  EmptyInput,
  InvalidFormat(String),
  InvalidNumber(String),
}

impl fmt::Display for ParseVersionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::EmptyInput => write!(f, "Sürüm metni boş olamaz."),
      Self::InvalidFormat(s) => write!(f, "Geçersiz SemVer formatı: '{}'", s),
      Self::InvalidNumber(s) => write!(f, "Sayısal olmayan sürüm basamağı: '{}'", s),
    }
  }
}
