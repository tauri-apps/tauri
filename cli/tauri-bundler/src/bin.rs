use tauri_bundler::{
  build_project,
  bundle::{
    bundle_project, check_icons, print_error, print_finished, PackageType, SettingsBuilder,
  },
};
pub use tauri_bundler::{Error, Result};

use clap::{crate_version, App, AppSettings, Arg, SubCommand};

#[cfg(windows)]
use runas::Command;
use std::env;

// Runs the CLI.
fn run() -> crate::Result<()> {
  let all_formats: Vec<&str> = PackageType::all()
    .iter()
    .map(PackageType::short_name)
    .collect();
  let m = App::new("cargo-tauri-bundler")
    .version(format!("v{}", crate_version!()).as_str())
    .bin_name("cargo")
    .setting(AppSettings::GlobalVersion)
    .setting(AppSettings::SubcommandRequired)
    .subcommand(
      SubCommand::with_name("tauri-bundler")
        .author("George Burton <burtonageo@gmail.com>, Lucas Fernandes Gonçalves Nogueira <lucas@tauri.studio>, Daniel Thompson-Yvetot <denjell@sfosc.org>, Tensor Programming <tensordeveloper@gmail.com>")
        .about("Bundle Rust executables into OS bundles")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
          Arg::with_name("bin")
            .long("bin")
            .value_name("NAME")
            .help("Bundle the specified binary"),
        )
        .arg(
          Arg::with_name("example")
            .long("example")
            .value_name("NAME")
            .conflicts_with("bin")
            .help("Bundle the specified example"),
        )
        .arg(
          Arg::with_name("format")
            .long("format")
            .value_name("FORMAT")
            .possible_values(&all_formats)
            .multiple(true)
            .help("Which bundle format to produce"),
        )
        .arg(
          Arg::with_name("release")
            .long("release")
            .help("Build a bundle from a target built in release mode"),
        )
        .arg(
          Arg::with_name("target")
            .long("target")
            .value_name("TRIPLE")
            .help("Build a bundle for the target triple"),
        )
        .arg(
          Arg::with_name("features")
            .long("features")
            .value_name("FEATURES")
            .multiple(true)
            .help("Which features to build"),
        )
        .arg(
          Arg::with_name("version")
            .long("version")
            .short("v")
            .help("Read the version of the bundler"),
        ),
    )
    .get_matches();

  if let Some(matches) = m.subcommand_matches("tauri-bundler") {
    if matches.is_present("version") {
      println!("{}", crate_version!());
    } else {
      let mut settings_builder = SettingsBuilder::new();
      if let Some(names) = matches.values_of("format") {
        let mut types = vec![];
        for name in names {
          match PackageType::from_short_name(name) {
            Some(package_type) => {
              types.push(package_type);
            }
            None => {
              return Err(crate::Error::GenericError(format!(
                "Unsupported bundle format: {}",
                name
              )));
            }
          }
        }
        settings_builder = settings_builder.package_types(types);
      }

      if let Some(triple) = matches.value_of("target") {
        settings_builder = settings_builder.target(triple.to_string());
      }
      if let Some(features) = matches.values_of_lossy("features") {
        settings_builder = settings_builder.features(features);
      }

      let output_paths = settings_builder
        .build()
        .and_then(|s| {
          if check_icons(&s)? {
            build_project(&s)?;
            Ok(s)
          } else {
            Err(crate::Error::IconPathError)
          }
        })
        .and_then(bundle_project)?;
      print_finished(&output_paths)?;
    }
  }
  Ok(())
}

fn main() {
  if let Err(error) = run() {
    print_error(&error.into()).expect("Failed to call print error in main");
  }
}
