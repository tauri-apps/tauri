use super::{DevProcess, MobileOptions};

use bossy::Handle;
use std::process::ExitStatus;

pub struct DevChild(Option<Handle>);

impl From<MobileOptions> for crate::mobile::CliOptions {
  fn from(options: MobileOptions) -> Self {
    Self {
      features: options.features,
      args: options.args,
      vars: Default::default(), // vars are filled on write
    }
  }
}

impl Drop for DevChild {
  fn drop(&mut self) {
    // consume the handle since we're not waiting on it
    // just to prevent a log error
    // note that this doesn't leak any memory
    self.0.take().unwrap().leak();
  }
}

impl DevProcess for DevChild {
  fn kill(&mut self) -> std::io::Result<()> {
    self
      .0
      .as_mut()
      .unwrap()
      .kill()
      .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to kill"))
  }

  fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
    self
      .0
      .as_mut()
      .unwrap()
      .try_wait()
      .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to wait"))
  }
}

pub mod android {
  use super::*;
  use crate::mobile::{android::run, write_options, Target};

  pub fn run_dev(
    options: MobileOptions,
    bundle_identifier: &str,
  ) -> crate::Result<impl DevProcess> {
    let debug = options.debug;
    write_options(options.into(), bundle_identifier, Target::Android)?;
    let handle = run(!debug)?;
    if handle.is_none() {
      // if the handle is None, we've opened Android Studio so we should just wait
      loop {
        std::thread::sleep(std::time::Duration::from_secs(60 * 60 * 24))
      }
    }
    Ok(DevChild(handle))
  }
}

#[cfg(target_os = "macos")]
pub mod ios {
  use super::*;
  use crate::mobile::{ios::run, write_options, Target};

  pub fn run_dev(
    options: MobileOptions,
    bundle_identifier: &str,
  ) -> crate::Result<impl DevProcess> {
    let debug = options.debug;
    write_options(options.into(), bundle_identifier, Target::Ios)?;
    let handle = run(!debug)?;
    if handle.is_none() {
      // if the handle is None, we've opened Xcode so we should just wait
      loop {}
    }
    Ok(DevChild(handle))
  }
}
