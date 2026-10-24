/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ops::{ControlFlow, FromResidual, Residual, Try};

use crate::Status::{Fail, Success};


#[derive(Debug, PartialEq)]
pub enum Status<T> { Fail, Success(T) }


#[macro_export]
macro_rules! ret_err {
  ($sum:expr, $msg:expr) => {{
    let msg = $msg;
    $sum.add(msg);

    return Status::Fail;
  }};
}

#[macro_export]
macro_rules! send_err {
  ($sum:expr, $msg:expr) => {{
    let msg = $msg;
    $sum.add(msg);
  }};
}


impl<T> Status<T> {
  pub fn is_success(&self) -> bool {
    matches!(self, Success(..))
  }

  pub fn is_fail(&self) -> bool {
    matches!(self, Fail)
  }


  pub fn expect(self, msg: &str) -> T {
    match self {
      Success(val) => val,
      Fail => panic!("{}", msg)
    }
  }
  
  pub fn unwrap(self) -> T {
    match self {
      Success(val) => val,
      Fail => panic!("called `Status::unwrap()` on a `Fail` value")
    }
  }
  
  pub fn unwrap_or(self, default: T) -> T {
    match self {
      Success(val) => val,
      Fail => default
    }
  }
  
  pub fn unwrap_or_else(self, default: impl FnOnce() -> T) -> T {
    match self {
      Success(val) => val,
      Fail => default()
    }
  }


  pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Status<O> {
    match self {
      Fail => Fail,
      Success(v) => Success(f(v))
    }
  }
  
  pub fn map_err(self, f: impl FnOnce()) -> Status<T> {
    match self {
      Fail => {f(); Fail},
      Success(val) => Success(val)
    }
  }
  
  pub fn inspect<O>(self, f: impl FnOnce(&T) -> O) -> Status<O> {
    match self {
      Fail => Fail,
      Success(v) => Success(f(&v))
    }
  }


  pub fn ok(self) -> Option<T> {
    match self {
      Fail => None,
      Success(v) => Some(v)
    }
  }

  pub fn ok_or<E>(self, e: E) -> Result<T, E> {
    match self {
      Success(val) => Ok(val),
      Fail => Err(e),
    }
  }

  pub fn ok_or_else<E>(self, f: impl FnOnce() -> E) -> Result<T, E> {
    match self {
      Success(val) => Ok(val),
      Fail => Err(f()),
    }
  }


  pub fn filter(self, f: impl FnOnce(&T) -> bool) -> Self {
    match self {
      Fail => Fail,
      Success(val) => if f(&val) { Success(val) } else { Fail }
    }
  }

}


impl<T, E> Status<Result<T, E>> {
  pub fn transpose(self) -> Result<Status<T>, E> {
    match self {
      Success(Ok(val)) => Ok(Success(val)),
      Success(Err(err)) => Err(err),
      Fail => Ok(Fail),
    }
  }
}

impl<T> Status<Option<T>> {
  pub fn transpose(self) -> Option<Status<T>> {
    match self {
      Success(Some(val)) => Some(Success(val)),
      Success(None) => None,
      Fail => Some(Fail),
    }
  }
}


pub trait StatusApi {
  type Output;

  fn transpose(self) -> Self::Output;
}

impl<T> StatusApi for Option<Status<T>> {
  type Output = Status<Option<T>>;

  fn transpose(self) -> Self::Output {
    match self {
      Some(Success(val)) => Success(Some(val)),
      Some(Fail) => Fail,
      None => Success(None),
    }
  }
}

impl<T, E> StatusApi for Result<Status<T>, E> {
  type Output = Status<Result<T, E>>;

  fn transpose(self) -> Self::Output {
    match self {
      Ok(Success(val)) => Success(Ok(val)),
      Ok(Fail) => Fail,
      Err(err) => Success(Err(err)),
    }
  }
}


pub trait StatusApi2 {
  type Output;

  fn status(self) -> Self::Output;
}

impl<T> StatusApi2 for Option<T> {
  type Output = Status<T>;

  fn status(self) -> Self::Output {
    match self {
      None => Fail,
      Some(v) => Success(v),
    }
  }
}

impl<T> StatusApi2 for Result<T, ()> {
  type Output = Status<T>;

  fn status(self) -> Self::Output {
    match self {
      Err(()) => Fail,
      Ok(v) => Success(v),
    }
  }
}



impl<T> Try for Status<T> {
  type Output = T;
  type Residual = Status<!>;
  
  fn from_output(output: Self::Output) -> Self {
    Status::Success(output)
  }

  fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
    match self {
      Status::Success(val) => ControlFlow::Continue(val),
      Status::Fail => ControlFlow::Break(Status::<!>::Fail),
    }
  }
}

impl<T> Residual<T> for Status<!> {
  type TryType = Status<T>;
}

impl<T> FromResidual<Status<!>> for Status<T> {
  fn from_residual(_residual: Status<!>) -> Self {
    Status::Fail
  }
}



pub trait CollectOrErrors: Iterator {
  fn collect_or_errors<T>(self) -> Status<Vec<T>>
  where
    Self: Sized + Iterator<Item = Status<T>>,
  {
    let mut vec = vec![];

    for item in self {
      match item {
        Status::Success(val) => vec.push(val),
        Status::Fail => return Status::Fail,
      }
    }

    Status::Success(vec)
  }
}

impl<I: Iterator> CollectOrErrors for I {}
