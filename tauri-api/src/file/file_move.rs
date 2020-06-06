use std::fs;
use std::path;

/// Moves a file from the given path to the specified destination.
///
/// `source` and `dest` must be on the same filesystem.
/// If `replace_using_temp` is specified, the destination file will be
/// replaced using the given temporary path.
///
/// * Errors:
///     * Io - copying / renaming
#[derive(Debug)]
pub struct Move<'a> {
  source: &'a path::Path,
  temp: Option<&'a path::Path>,
}
impl<'a> Move<'a> {
  /// Specify source file
  pub fn from_source(source: &'a path::Path) -> Move<'a> {
    Self { source, temp: None }
  }

  /// If specified and the destination file already exists, the "destination"
  /// file will be moved to the given temporary location before the "source"
  /// file is moved to the "destination" file.
  ///
  /// In the event of an `io` error while renaming "source" to "destination",
  /// the temporary file will be moved back to "destination".
  ///
  /// The `temp` dir must be explicitly provided since `rename` operations require
  /// files to live on the same filesystem.
  pub fn replace_using_temp(&mut self, temp: &'a path::Path) -> &mut Self {
    self.temp = Some(temp);
    self
  }

  /// Move source file to specified destination
  pub fn to_dest(&self, dest: &path::Path) -> crate::Result<()> {
    match self.temp {
      None => {
        fs::rename(self.source, dest)?;
      }
      Some(temp) => {
        if dest.exists() {
          fs::rename(dest, temp)?;
          if let Err(e) = fs::rename(self.source, dest) {
            fs::rename(temp, dest)?;
            return Err(e.into());
          }
        } else {
          fs::rename(self.source, dest)?;
        }
      }
    };
    Ok(())
  }
}
