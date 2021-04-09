// SPDX-License-Identifier: Apache-2.0 OR MIT

use colored::Colorize;

pub struct Logger<'a> {
  context: &'a str,
}

impl<'a> Logger<'a> {
  pub fn new(context: &'a str) -> Self {
    Self { context }
  }

  pub fn log(&self, message: impl AsRef<str>) {
    println!(
      "{} {}",
      format!("[{}]", self.context).green().bold(),
      message.as_ref()
    );
  }

  pub fn warn(&self, message: impl AsRef<str>) {
    println!(
      "{} {}",
      format!("[{}]", self.context).yellow().bold(),
      message.as_ref()
    );
  }

  pub fn error(&self, message: impl AsRef<str>) {
    println!(
      "{} {}",
      format!("[{}]", self.context).red().bold(),
      message.as_ref()
    );
  }
}
