#[macro_use]
extern crate clap;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate tempfile;

mod bundle;

use crate::bundle::{bundle_project, BuildArtifact, PackageType, Settings};
use clap::{App, AppSettings, Arg, SubCommand};
use std::env;
use std::process;

error_chain! {
  foreign_links {
        Glob(::glob::GlobError);
        GlobPattern(::glob::PatternError);
        Io(::std::io::Error);
        Image(::image::ImageError);
        Target(::target_build_utils::Error);
        Term(::term::Error);
        Toml(::toml::de::Error);
        Walkdir(::walkdir::Error);
        StripError(std::path::StripPrefixError);
    }
    errors {}
}

/// Runs `cargo build` to make sure the binary file is up-to-date.
fn build_project_if_unbuilt(settings: &Settings) -> crate::Result<()> {
  let mut args = vec!["build".to_string()];
  if let Some(triple) = settings.target_triple() {
    args.push(format!("--target={}", triple));
  }
  match settings.build_artifact() {
    &BuildArtifact::Main => {}
    &BuildArtifact::Bin(ref name) => {
      args.push(format!("--bin={}", name));
    }
    &BuildArtifact::Example(ref name) => {
      args.push(format!("--example={}", name));
    }
  }
  if settings.is_release_build() {
    args.push("--release".to_string());
  }

  match settings.build_features() {
    Some(features) => {
      args.push(format!("--features={}", features.join(" ")));
    }
    None => {}
  }

  let status = process::Command::new("cargo").args(args).status()?;
  if !status.success() {
    bail!(
      "Result of `cargo build` operation was unsuccessful: {}",
      status
    );
  }
  Ok(())
}

fn run() -> crate::Result<()> {
  let all_formats: Vec<&str> = PackageType::all()
    .iter()
    .map(PackageType::short_name)
    .collect();
  let m = App::new("cargo-tauri-cli")
    .version(format!("v{}", crate_version!()).as_str())
    .bin_name("cargo")
    .setting(AppSettings::GlobalVersion)
    .setting(AppSettings::SubcommandRequired)
    .subcommand(
      SubCommand::with_name("tauri-cli")
        .author("George Burton <burtonageo@gmail.com>, Lucas Fernandes Gonçalves Nogueira <lucas@quasar.dev>, Daniel Thompson-Yvetot <denjell@sfosc.org>")
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
            .help("Read the version of the CLI"),
        ),
    )
    .get_matches();

  if let Some(m) = m.subcommand_matches("tauri-cli") {
    if m.is_present("version") {
      println!("{}", crate_version!());
    } else {
      let output_paths = env::current_dir()
        .map_err(From::from)
        .and_then(|d| Settings::new(d, m))
        .and_then(|s| {
          build_project_if_unbuilt(&s)?;
          Ok(s)
        })
        .and_then(bundle_project)?;
      bundle::print_finished(&output_paths)?;
    }
  }
  Ok(())
}

fn main() {
  if let Err(error) = run() {
    bundle::print_error(&error).unwrap();
  }
}
