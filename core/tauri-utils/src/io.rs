// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! IO helpers.

use std::{fs::read_to_string, io::BufRead, path::Path};

/// Write to the given file if the content is different.
pub fn write_if_changed(content: &str, path: &Path) -> std::io::Result<()> {
  if content != read_to_string(path).unwrap_or_default() {
    std::fs::write(path, content)
  } else {
    Ok(())
  }
}

/// Read all bytes until a newline (the `0xA` byte) or a carriage return (`\r`) is reached, and append them to the provided buffer.
///
/// Adapted from <https://doc.rust-lang.org/std/io/trait.BufRead.html#method.read_line>.
pub fn read_line<R: BufRead + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> std::io::Result<usize> {
  let mut read = 0;
  loop {
    let (done, used) = {
      let available = match r.fill_buf() {
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,

        Err(e) => return Err(e),
      };
      match memchr::memchr(b'\n', available) {
        Some(i) => {
          let end = i + 1;
          buf.extend_from_slice(&available[..end]);
          (true, end)
        }
        None => match memchr::memchr(b'\r', available) {
          Some(i) => {
            let end = i + 1;
            buf.extend_from_slice(&available[..end]);
            (true, end)
          }
          None => {
            buf.extend_from_slice(available);
            (false, available.len())
          }
        },
      }
    };
    r.consume(used);
    read += used;
    if done || used == 0 {
      return Ok(read);
    }
  }
}
