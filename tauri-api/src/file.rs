use std::fs;

mod extract;
mod file_move;

use crate::{Error, ErrorKind};
pub use extract::*;
pub use file_move::*;

pub fn read_string(file: String) -> crate::Result<String> {
  fs::read_to_string(file)
    .map_err(|err| Error::from(ErrorKind::File(format!("Read_string failed: {}", err))))
    .map(|c| c)
}

pub fn read_binary(file: String) -> crate::Result<Vec<u8>> {
  fs::read(file)
    .map_err(|err| Error::from(ErrorKind::File(format!("Read_binary failed: {}", err))))
    .map(|b| b)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{Error, ErrorKind};
  use totems::{assert_err, assert_ok};

  #[test]
  fn check_read_string() {
    let file = String::from("test/test.txt");

    let res = read_string(file);

    assert_ok!(res);

    if let Ok(s) = res {
      assert_eq!(s, "This is a test doc!".to_string());
    }
  }

  #[test]
  fn check_read_string_fail() {
    let file = String::from("test/");

    let res = read_string(file);

    assert_err!(res);

    if let Err(Error(ErrorKind::File(e), _)) = res {
      assert_eq!(
        e,
        "Read_string failed: Access is denied. (os error 5)".to_string()
      );
    }
  }

  #[test]
  fn check_read_binary() {}
}
