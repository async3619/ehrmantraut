use std::fmt;

#[derive(Debug)]
pub enum LowerError {
  UnexpectedNode { kind: String, expected: String },
}

impl fmt::Display for LowerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LowerError::UnexpectedNode { kind, expected } => {
        write!(f, "unexpected node '{kind}', expected {expected}")
      }
    }
  }
}

impl std::error::Error for LowerError {}
