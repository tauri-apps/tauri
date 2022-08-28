use super::{detect_target_ok, ensure_init, env, init_dot_cargo, with_config, MobileTarget};
use crate::Result;
use clap::Parser;

use cargo_mobile::{
  android::target::Target,
  opts::Profile,
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

  with_config(None, |app, config, metadata, cli_options| {
    ensure_init(config.project_dir(), MobileTarget::Android)?;

    let env = env()?;
    init_dot_cargo(app, Some((&env, config)))?;

    call_for_targets_with_fallback(
      options.targets.unwrap_or_default().iter(),
      &detect_target_ok,
      &env,
      |target: &Target| {
        target
          .build(
            config,
            metadata,
            &env,
            cli_options.noise_level,
            true,
            profile,
          )
          .map_err(Into::into)
      },
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?
  })
}
