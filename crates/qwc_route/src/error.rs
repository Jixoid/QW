use core::fmt;
use std::{assert_matches, ops::BitOr};


#[derive(Debug)]
pub enum Error {
  New,
  Io(std::io::Error),
  Ds(qwc_ds::Error),
  Str(String),
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self { Error::Io(err) }
}

impl From<qwc_ds::Error> for Error {
  fn from(err: qwc_ds::Error) -> Self { Error::Ds(err) }
}

impl From<String> for Error {
  fn from(err: String) -> Self { Error::Str(err) }
}

impl From<&str> for Error {
  fn from(err: &str) -> Self { Error::Str(err.to_string()) }
}


impl BitOr<String> for Error {
  type Output = Error;
  
  fn bitor(self, rhs: String) -> Self::Output {
    assert_matches!(self, Self::New);
    Self::from(rhs)
  }
}

impl BitOr<&str> for Error {
  type Output = Error;
  
  fn bitor(self, rhs: &str) -> Self::Output {
    assert_matches!(self, Self::New);
    Self::from(rhs)
  }
}



impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::New => panic!(),
      Error::Io(e) => write!(f, "io: {}", e),
      Error::Ds(e) => write!(f, "ds: {}", e),
      Error::Str(v) => write!(f, "{}", v),
    }
  }
}
