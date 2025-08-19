// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{helpers::template, Result};
use anyhow::Context;
use cargo_mobile2::{config::app::App, open_harmony::config::Config, os, util};
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};

use std::path::Path;

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/mobile/open-harmony");

pub fn gen(
  app: &App,
  config: &Config,
  (handlebars, mut map): (Handlebars, template::JsonMap),
) -> Result<()> {
  println!("Generating DevEco Studio project...");
  let dest = config.project_dir();

  map.insert(
    "root-dir-rel",
    Path::new(&os::replace_path_separator(
      util::relativize_path(app.root_dir(), dest.join(app.name_snake())).into_os_string(),
    )),
  );
  map.insert("root-dir", app.root_dir());
  map.insert("windows", cfg!(windows));

  template::render(&handlebars, map.inner(), &TEMPLATE_DIR, &dest)
    .with_context(|| "failed to process template")?;

  Ok(())
}
