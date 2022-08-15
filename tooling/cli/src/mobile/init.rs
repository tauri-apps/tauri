// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use crate::helpers::mobile::init::Target;
use crate::helpers::{app_paths::tauri_dir, mobile::init::*};
use crate::Result;
use cargo_mobile::util::cli::TextWrapper;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "Initializes a Tauri Android project")]
pub struct Options {
  /// Skip prompting for values
  #[clap(long)]
  ci: bool,
}

pub fn command(mut options: Options, target: Target) -> Result<()> {
  options.ci = options.ci || std::env::var("CI").is_ok();

  let wrapper = TextWrapper::with_splitter(textwrap::termwidth(), textwrap::NoHyphenation);
  exec(
    target,
    &wrapper,
    options.ci.into(),
    SkipDevTools::No,
    ReinstallDeps::Yes,
    OpenInEditor::No,
    tauri_dir(),
  )
  .map_err(|e| anyhow::anyhow!("{:#}", e))?;
  Ok(())
}
