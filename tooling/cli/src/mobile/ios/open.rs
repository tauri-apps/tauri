use super::{ensure_init, env, with_config, MobileTarget};
use crate::Result;
use cargo_mobile::os;

pub fn command() -> Result<()> {
  with_config(
    Some(Default::default()),
    |_root_conf, config, _metadata, _cli_options| {
      ensure_init(config.project_dir(), MobileTarget::Ios)?;
      let env = env()?;
      os::open_file_with("Xcode", config.project_dir(), &env).map_err(Into::into)
    },
  )
}
