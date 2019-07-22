use semver;
use std;

#[derive(Debug)]
pub enum Error {
  SemVer(semver::SemVerError),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Error::*;
    match *self {
      SemVer(ref e) => write!(f, "SemVerError: {}", e),
    }
  }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    "Version Error"
  }

  fn cause(&self) -> Option<&dyn std::error::Error> {
    use Error::*;
    Some(match *self {
      SemVer(ref e) => e,
    })
  }
}

impl From<semver::SemVerError> for Error {
  fn from(e: semver::SemVerError) -> Self {
    Error::SemVer(e)
  }
}
