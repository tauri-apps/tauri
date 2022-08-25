use super::{detect_target_ok, ensure_init, env, init_dot_cargo, with_config, Error, MobileTarget};
use crate::Result;
use clap::Parser;

use cargo_mobile::{
  android::target::Target,
  opts::{NoiseLevel, Profile},
  target::{call_for_targets_with_fallback, TargetTrait},
};

#[derive(Debug, Parser)]
pub struct Options {
  /// Targets to build.
  #[clap(
    short,
    long = "target",
    multiple_occurrences(true),
    multiple_values(true),
    default_value = Target::DEFAULT_KEY,
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  targets: Option<Vec<String>>,
  /// Builds with the release flag
  #[clap(short, long)]
  release: bool,
}

pub fn command(options: Options) -> Result<()> {
  let profile = if options.release {
    Profile::Release
  } else {
    Profile::Debug
  };
  let noise_level = NoiseLevel::FranklyQuitePedantic;

  with_config(None, |root_conf, config, metadata| {
    ensure_init(config.project_dir(), MobileTarget::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;

    let env = env()?;
    init_dot_cargo(root_conf, Some(&env)).map_err(Error::InitDotCargo)?;

    call_for_targets_with_fallback(
      options.targets.unwrap_or_default().iter(),
      &detect_target_ok,
      &env,
      |target: &Target| {
        target
          .build(config, metadata, &env, noise_level, true, profile)
          .map_err(Error::AndroidStudioScriptFailed)
      },
    )
    .map_err(|e| Error::TargetInvalid(e.to_string()))?
  })
  .map_err(Into::into)
}
