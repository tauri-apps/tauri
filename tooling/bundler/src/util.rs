use std::path::{Path, PathBuf};

pub fn display_path<P: AsRef<Path>>(p: P) -> String {
  p.as_ref()
    .components()
    .collect::<PathBuf>()
    .to_string_lossy()
    .to_string()
}
