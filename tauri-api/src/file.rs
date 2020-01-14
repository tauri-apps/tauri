use std::fs;

mod extract;
mod file_move;

use crate::Error;
pub use extract::*;
pub use file_move::*;

pub fn read_string(file: String) -> crate::Result<String> {
  fs::read_to_string(file)
    .map_err(|err| Error::with_chain(err, "read string failed"))
    .map(|c| c)
}

pub fn read_binary(file: String) -> crate::Result<Vec<u8>> {
  fs::read(file)
    .map_err(|err| Error::with_chain(err, "Read binary failed"))
    .map(|b| b)
}
