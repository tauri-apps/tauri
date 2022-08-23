use super::{ensure_init, with_config, Error, MobileTarget};
use crate::Result;
use cargo_mobile::os;

pub fn command() -> Result<()> {
  with_config(|_, config, _metadata| {
    ensure_init(config.project_dir(), MobileTarget::Ios)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    os::open_file_with("Xcode", config.project_dir()).map_err(Error::OpenFailed)
  })
  .map_err(Into::into)
}
