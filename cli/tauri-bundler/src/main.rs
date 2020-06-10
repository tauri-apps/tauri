mod bundle;
mod error;
mod sign;

pub use error::{Error, Result};

use crate::bundle::{bundle_project, check_icons, BuildArtifact, PackageType, Settings};

#[cfg(windows)]
use crate::bundle::print_info;

use crate::sign::{generate_key, save_keypair};

use clap::{crate_version, App, AppSettings, Arg, SubCommand};
#[cfg(windows)]
use runas::Command;
use std::env;
use std::path::Path;
use std::process;

const AUTHORS: &str = "George Burton <burtonageo@gmail.com>, Lucas Fernandes Gonçalves Nogueira <lucas@tauri.studio>, Daniel Thompson-Yvetot <denjell@sfosc.org>, Tensor Programming <tensordeveloper@gmail.com>, David Lemarier <david@lemarier.ca>";

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
    return Err(crate::Error::GenericError(format!(
      "Result of `cargo build` operation was unsuccessful: {}",
      status
    )));
  }
  Ok(())
}

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
        .author(AUTHORS)
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
    .subcommand(
      SubCommand::with_name("tauri-sign-updates")
        .author(AUTHORS)
        .about("Sign executables for tauri updater.")
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
          Arg::with_name("sign")
            .long("sign")
            .short("s")
            .value_name("PATH")
            .help("Sign the specified binary"),
        )
        .arg(
          Arg::with_name("private-key-path")
            .short("p")
            .long("private-key-path")
            .conflicts_with("private-key")
            .value_name("PATH")
            .requires("sign")
            .help("Load the private key from a file"),
        )
        .arg(
          Arg::with_name("private-key")
            .long("private-key")
            .value_name("STRING")
            .requires("sign")
            .help("Load the private key from a string"),
        )
        .arg(
          Arg::with_name("generate-key")
            .long("generate-key")
            .short("g")
            .help("Generate keypair to sign files"),
        )
        .arg(
          Arg::with_name("password")
            .long("password")
            .short("P")
            .requires("sign")
            .value_name("PASSWORD")
            .help("Set private key password when signing"),
        )
        .arg(
          Arg::with_name("write-private-key")
            .long("write-private-key")
            .short("w")
            .requires("generate-key")
            .value_name("DESTINATION")
            .help("Write private key to a file"),
        )
        .arg(
          Arg::with_name("no-password")
            .long("no-password")
            .requires("generate-key")
            .conflicts_with("password")
            .help("Set empty password for your private key"),
        )
        .arg(
          Arg::with_name("force")
            .long("force")
            .short("f")
            .requires_all(&["generate-key", "write-private-key"])
            .help("Overwrite private key even if exist on the specified path"),
        ),
    )
    .get_matches();

  #[cfg(windows)]
  {
    print_info("Running Loopback command")?;
    Command::new("cmd")
      .args(&vec![
        "CheckNetIsolation.exe",
        r#"LoopbackExempt -a -n="Microsoft.Win32WebViewHost_cw5n1h2txyewy""#,
      ])
      .force_prompt(true);
  }

  // Bundler
  if let Some(m) = m.subcommand_matches("tauri-bundler") {
    if m.is_present("version") {
      println!("{}", crate_version!());
    } else {
      let output_paths = env::current_dir()
        .map_err(From::from)
        .and_then(|d| Settings::new(d, m))
        .and_then(|s| {
          if check_icons(&s)? {
            build_project_if_unbuilt(&s)?;
            Ok(s)
          } else {
            Err(crate::Error::IconPathError)
          }
        })
        .and_then(bundle_project)?;
      bundle::print_finished(&output_paths)?;
    }
  }

  // Sign files
  if let Some(m) = m.subcommand_matches("tauri-sign-updates") {
    if m.is_present("generate-key") {
      let mut secret_key_password: Option<String> = None;

      // If no password flag provided, we use empty string
      // If we send None to minisign they'll prompt for the password
      if m.is_present("no-password") {
        secret_key_password = Some("".to_string());
      }

      let keypair = generate_key(secret_key_password)?;

      if m.is_present("write-private-key") {
        let mut force = false;
        if m.is_present("force") {
          force = true
        }

        let (secret_path, public_path) = save_keypair(
          force,
          Path::new(&m.value_of("write-private-key").unwrap()),
          &keypair.sk,
          &keypair.pk,
        )?;
        println!(
        "\nYour keypair was generated successfully\nPrivate: {} (Keep it secret!)\nPublic: {}\n---------------------------",
        secret_path.display(),
        public_path.display()
        )
      } else {
        println!(
          "\nYour secret key was generated successfully - Keep it secret!\n{}\n\n",
          keypair.sk
        );
        println!(
          "Your public key was generated successfully:\n{}\n\nAdd this in your tauri.conf.json",
          keypair.pk
        );
      }

      println!("\nATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not works.\n---------------------------\n");
    } else if m.is_present("sign") {
      // get the priv key password
      let mut secret_key_password: Option<String>;
      let mut private_key: Option<String> = None;

      // If no password flag provided, we use empty string
      // If we send None to minisign they'll prompt for the password
      if m.is_present("password") {
        secret_key_password = Some(m.value_of("password").unwrap().to_string());
      } else {
        // check env variable
        match env::var_os("TAURI_KEY_PASSWORD") {
          Some(value) => {
            secret_key_password = Some(String::from(value.to_str().unwrap()));
          }
          None => secret_key_password = Some("".to_string()),
        }
      }

      // get the priv key as a string
      if m.is_present("private-key-path") {
        private_key = Some(sign::read_key_from_file(Path::new(
          &m.value_of("private-key-path").unwrap(),
        ))?);
      } else if m.is_present("private-key") {
        private_key = Some(m.value_of("private-key").unwrap().to_string());
      }

      // we need to provide at least an empty string...
      if secret_key_password.is_none() {
        secret_key_password = Some("".to_string());
      }

      if private_key.is_none() {
        return Err(crate::Error::GenericError(
          "Key generation aborted: Unable to find the private key".to_string(),
        ));
      }

      let (manifest_dir, signature) = sign::sign_file(
        private_key.unwrap(),
        secret_key_password.unwrap(),
        Path::new(&m.value_of("sign").unwrap()),
        false,
      )?;

      println!(
        "\nYour file was signed successfully, You can find the signature here:\n{}\n\nPublic signature:\n{}\n\nMake sure to include this into the signature field of your update server.",
        manifest_dir.display(),
        signature
      );
    };
  }

  Ok(())
}

fn main() {
  if let Err(error) = run() {
    bundle::print_error(&error.into()).expect("Failed to call print error in main");
  }
}
