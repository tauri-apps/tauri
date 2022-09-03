// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::BTreeMap,
  fs::{create_dir_all, File},
  io::Write,
  path::Path,
};

use handlebars::Handlebars;
use include_dir::Dir;

pub fn render<P: AsRef<Path>>(
  handlebars: &Handlebars<'_>,
  data: &BTreeMap<&str, serde_json::Value>,
  dir: &Dir<'_>,
  out_dir: P,
) -> crate::Result<()> {
  create_dir_all(out_dir.as_ref().join(dir.path()))?;
  for file in dir.files() {
    let mut file_path = file.path().to_path_buf();
    // cargo for some reason ignores the /templates folder packaging when it has a Cargo.toml file inside
    // so we rename the extension to `.crate-manifest`
    if let Some(extension) = file_path.extension() {
      if extension == "crate-manifest" {
        file_path.set_extension("toml");
      }
    }
    let mut output_file = File::create(out_dir.as_ref().join(file_path))?;
    if let Some(utf8) = file.contents_utf8() {
      handlebars
        .render_template_to_write(utf8, &data, &mut output_file)
        .expect("Failed to render template");
    } else {
      output_file.write_all(file.contents())?;
    }
  }
  for dir in dir.dirs() {
    render(handlebars, data, dir, out_dir.as_ref())?;
  }
  Ok(())
}
