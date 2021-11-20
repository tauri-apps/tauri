// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::ArgMatches;

mod generate;
mod sign;

pub fn command(matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("generate") {
    generate::command(matches)?;
  } else if let Some(matches) = matches.subcommand_matches("sign") {
    sign::command(matches)?;
  }
  Ok(())
}
