use core::fmt;


#[derive(Debug)]
pub enum CompilerError {
  Io(std::io::Error),
  Ds(String),
  Str(String),
}

impl From<std::io::Error> for CompilerError {
  fn from(err: std::io::Error) -> Self {
    CompilerError::Io(err)
  }
}

impl From<String> for CompilerError {
  fn from(err: String) -> Self {
    CompilerError::Str(err)
  }
}

impl fmt::Display for CompilerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CompilerError::Io(e)  => write!(f, "io: {}", e),
      CompilerError::Ds(e) => write!(f, "ds: {}", e),
      _ => write!(f, "unkown"),
    }
  }
}

pub type Result<T> = std::result::Result<T, CompilerError>;
