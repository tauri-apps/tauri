use super::{DevProcess, Options};

use bossy::Handle;
use std::process::ExitStatus;

pub struct DevChild(Option<Handle>);

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
  use crate::mobile::android::run;

  pub fn run_dev(options: Options) -> crate::Result<impl DevProcess> {
    let handle = run(!options.debug)?;
    Ok(DevChild(Some(handle)))
  }
}

#[cfg(target_os = "macos")]
pub mod ios {
  use super::*;

  pub fn run_dev() -> crate::Result<impl DevProcess> {
    todo!()
  }
}
