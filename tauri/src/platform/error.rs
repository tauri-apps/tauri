use std;

#[derive(Debug)]
pub enum Error {
  Arch(String),
  Target(String),
  Abi(String),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Error::*;
    match *self {
      Arch(ref s) => write!(f, "ArchError: {}", s),
      Target(ref e) => write!(f, "TargetError: {}", e),
      Abi(ref e) => write!(f, "AbiError: {}", e),
    }
  }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    "Platform Error"
  }

  fn cause(&self) -> Option<&dyn std::error::Error> {
    return None;
  }
}
