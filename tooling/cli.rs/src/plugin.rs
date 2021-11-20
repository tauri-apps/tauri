// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::ArgMatches;

use crate::Result;

mod init;

pub fn command(matches: &ArgMatches) -> Result<()> {
  if let Some(matches) = matches.subcommand_matches("init") {
    init::command(matches)?;
  }

  Ok(())
}
