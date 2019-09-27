use std::fs::{create_dir, create_dir_all, read_dir, remove_dir_all};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Options {
  pub overwrite: bool,
  pub skip: bool,
  pub buffer_size: usize,
  pub copy_files: bool,
  pub content_only: bool,
  pub depth: u64,
}

impl Options {
  pub fn new() -> Options {
    Options {
      overwrite: false,
      skip: false,
      buffer_size: 64000,
      copy_files: false,
      content_only: false,
      depth: 0,
    }
  }
}
