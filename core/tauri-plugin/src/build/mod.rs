// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env::current_dir,
  fs::create_dir_all,
  path::{Path, PathBuf},
};

use cargo_metadata::{Metadata, MetadataCommand};
use tauri_utils::acl::{self, Error};

pub struct Builder<'a> {
  commands: &'a [&'static str],
}

impl<'a> Builder<'a> {
  pub fn new(commands: &'a [&'static str]) -> Self {
    Self { commands }
  }

  /// [`Self::try_build`] but will exit automatically if an error is found.
  pub fn build(self) {
    if let Err(error) = self.try_build() {
      println!("{}: {}", env!("CARGO_PKG_NAME"), error);
      std::process::exit(1);
    }
  }

  /// Ensure this crate is properly configured to be a Tauri plugin.
  ///
  /// # Errors
  ///
  /// Errors will occur if environmental variables expected to be set inside of [build scripts]
  /// are not found, or if the crate violates Tauri plugin conventions.
  pub fn try_build(self) -> Result<(), Error> {
    // convention: plugin names should not use underscores
    let name = build_var("CARGO_PKG_NAME")?;
    if name.contains('_') {
      return Err(Error::CrateName);
    }

    // requirement: links MUST be set and MUST match the name
    let _links = build_var("CARGO_MANIFEST_LINKS")?;

    let autogenerated = Path::new("permissions/autogenerated/");
    let commands_dir = &autogenerated.join("commands");

    if !self.commands.is_empty() {
      autogenerate_command_permissions(commands_dir, self.commands);
    }

    let permissions = acl::build::define_permissions("./permissions/**/*.*")?;
    acl::build::generate_schema(&permissions)?;

    let metadata = find_metadata()?;
    println!("{metadata:#?}");

    Ok(())
  }
}

fn autogenerate_command_permissions(path: &Path, commands: &[&str]) {
  if !path.exists() {
    create_dir_all(path).expect("unable to create autogenerated commands dir");
  }

  let cwd = current_dir().unwrap();
  let components_len = path.strip_prefix(&cwd).unwrap_or(path).components().count();
  let schema_path = (1..components_len)
    .map(|_| "..")
    .collect::<PathBuf>()
    .join(acl::build::PERMISSION_SCHEMA_FILE_NAME);

  for command in commands {
    let slugified_command = command.replace('_', "-");
    let toml = format!(
      r###"# Automatically generated - DO NOT EDIT!

"$schema" = "{schema_path}"

[[permission]]
identifier = "allow-{slugified_command}"
description = "Enables the {command} command without any pre-configured scope."
commands.allow = ["{command}"]

[[permission]]
identifier = "deny-{slugified_command}"
description = "Denies the {command} command without any pre-configured scope."
commands.deny = ["{command}"]
"###,
      command = command,
      slugified_command = slugified_command,
      schema_path = schema_path.display()
    );

    std::fs::write(path.join(format!("{command}.toml")), toml)
      .unwrap_or_else(|_| panic!("unable to autogenerate ${command}.toml"));
  }
}

/// Grab an env var that is expected to be set inside of build scripts.
fn build_var(key: &str) -> Result<String, Error> {
  std::env::var(key).map_err(|_| Error::BuildVar(key.into()))
}

fn find_metadata() -> Result<Metadata, Error> {
  build_var("CARGO_MANIFEST_DIR").and_then(|dir| {
    MetadataCommand::new()
      .current_dir(dir)
      .no_deps()
      .exec()
      .map_err(Error::Metadata)
  })
}
