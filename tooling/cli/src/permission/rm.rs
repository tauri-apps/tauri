use clap::Parser;

use crate::Result;

#[derive(Debug, Parser)]
#[clap(
  about = "Remove a permission from `acl` directory, `tauri.conf.json` and `permissions.json`"
)]
pub struct Options {
  /// Permission to remove.
  identifier: String,
}

pub fn command(options: Options) -> Result<()> {
  Ok(())
}
