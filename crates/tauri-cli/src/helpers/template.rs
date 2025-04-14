// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs::{create_dir_all, File},
  io::Write,
  path::{Path, PathBuf},
};

use handlebars::{to_json, Handlebars};
use include_dir::Dir;
use serde::Serialize;
use serde_json::value::{Map, Value as JsonValue};

/// Map of template variable names and values.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct JsonMap(Map<String, JsonValue>);

impl Default for JsonMap {
  fn default() -> Self {
    Self(Map::new())
  }
}

impl JsonMap {
  pub fn insert(&mut self, name: &str, value: impl Serialize) {
    self.0.insert(name.to_owned(), to_json(value));
  }

  pub fn inner(&self) -> &Map<String, JsonValue> {
    &self.0
  }
}

pub fn render<P: AsRef<Path>, D: Serialize>(
  handlebars: &Handlebars<'_>,
  data: &D,
  dir: &Dir<'_>,
  out_dir: P,
) -> crate::Result<()> {
  let out_dir = out_dir.as_ref();
  let mut created_dirs = Vec::new();
  render_with_generator(handlebars, data, dir, out_dir, &mut |file_path: PathBuf| {
    let path = out_dir.join(file_path);
    let parent = path.parent().unwrap().to_path_buf();
    if !created_dirs.contains(&parent) {
      create_dir_all(&parent)?;
      created_dirs.push(parent);
    }
    File::create(path).map(Some)
  })
}

pub fn render_with_generator<
  P: AsRef<Path>,
  D: Serialize,
  F: FnMut(PathBuf) -> std::io::Result<Option<File>>,
>(
  handlebars: &Handlebars<'_>,
  data: &D,
  dir: &Dir<'_>,
  out_dir: P,
  out_file_generator: &mut F,
) -> crate::Result<()> {
  let out_dir = out_dir.as_ref();
  for file in dir.files() {
    let mut file_path = file.path().to_path_buf();
    // cargo for some reason ignores the /templates folder packaging when it has a Cargo.toml file inside
    // so we rename the extension to `.crate-manifest`
    if let Some(extension) = file_path.extension() {
      if extension == "crate-manifest" {
        file_path.set_extension("toml");
      }
    }
    if let Some(mut output_file) = out_file_generator(file_path)? {
      if let Some(utf8) = file.contents_utf8() {
        handlebars
          .render_template_to_write(utf8, &data, &mut output_file)
          .expect("Failed to render template");
      } else {
        output_file.write_all(file.contents())?;
      }
    }
  }
  for dir in dir.dirs() {
    render_with_generator(handlebars, data, dir, out_dir, out_file_generator)?;
  }
  Ok(())
}
