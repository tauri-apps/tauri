use super::{ensure_init, with_config, Error, MobileTarget};
use crate::Result;
use cargo_mobile::os;

pub fn command() -> Result<()> {
  with_config(
    Some(Default::default()),
    |_root_conf, config, _metadata, _cli_options| {
      ensure_init(config.project_dir(), MobileTarget::Android)
        .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
      os::open_file_with("Android Studio", config.project_dir()).map_err(Error::OpenFailed)
    },
  )
  .map_err(Into::into)
}
