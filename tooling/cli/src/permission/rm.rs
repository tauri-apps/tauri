use clap::Parser;

use crate::Result;

#[derive(Debug, Parser)]
#[clap(about = "Add a tauri plugin to the project")]
pub struct Options {}

pub fn command(options: Options) -> Result<()> {
  Ok(())
}
