#![allow(missing_docs)]
/// TODO: proper documentation for this module
pub use phf;
use std::borrow::{Borrow, Cow};
use std::fmt;
use std::io::{self, Cursor, Error, ErrorKind, Read};

use flate2::bufread::GzDecoder;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Compression {
  None,
  Gzip,
}

impl fmt::Display for Compression {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    match *self {
      Compression::None => fmt.write_str("None"),
      Compression::Gzip => fmt.write_str("Gzip"),
    }
  }
}

/// Runtime access to the included files
pub struct Assets {
  #[doc(hidden)]
  pub files: phf::Map<&'static str, (Compression, &'static [u8])>,
}

#[cfg(windows)]
fn as_key(path: &str) -> Cow<'_, str> {
  Cow::Owned(path.replace("\\", "/"))
}

#[cfg(not(windows))]
fn as_key(path: &str) -> Cow<'_, str> {
  Cow::Borrowed(path)
}

impl Assets {
  pub fn file_names(&'static self) -> FileNames {
    FileNames {
      iter: self.files.keys(),
    }
  }

  pub fn is_available(&self, path: &str) -> bool {
    self.files.contains_key(path)
  }

  pub fn get(&self, path: &str) -> io::Result<Cow<'static, [u8]>> {
    match self.get_raw(path) {
      Ok((Compression::None, data)) => Ok(data),
      Ok((Compression::Gzip, compressed)) => {
        let mut r = GzDecoder::new(Cursor::new(compressed));
        let mut v = Vec::new();
        r.read_to_end(&mut v)?;
        Ok(Cow::Owned(v))
      }
      Err(e) => Err(e),
    }
  }

  pub fn get_raw(&self, path: &str) -> io::Result<(Compression, Cow<'static, [u8]>)> {
    let key = as_key(path);

    self
      .files
      .get(&*key)
      .map(|&(c, d)| (c, Cow::Owned(d.to_owned())))
      .ok_or_else(|| Error::new(ErrorKind::NotFound, "Key not found"))
  }

  pub fn read(&self, path: &str) -> io::Result<Box<dyn Read>> {
    let key = as_key(path);
    match self.files.get(key.borrow() as &str) {
      Some(b) => match b.0 {
        Compression::None => Ok(Box::new(Cursor::new(b.1))),
        Compression::Gzip => Ok(Box::new(GzDecoder::new(Cursor::new(b.1)))),
      },
      None => Err(Error::new(ErrorKind::NotFound, "Key not found")),
    }
  }
}

/// Iterates over the file names available for `Files` object.
pub struct FileNames {
  // Our internal iterator.  We wrap this in a nice struct so our
  // caller doesn't need to know the details.
  iter: phf::map::Keys<'static, &'static str, (Compression, &'static [u8])>,
}

impl Iterator for FileNames {
  type Item = &'static str;

  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().cloned()
  }
}
