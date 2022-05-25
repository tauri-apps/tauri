// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to CLI arguments.

use crate::{
  utils::config::{CliArg, CliConfig},
  PackageInfo,
};

use clap::{Arg, ArgMatches, ErrorKind};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[macro_use]
mod macros;

mod clapfix {
  //! Compatibility between `clap` 3.0 and 3.1+ without deprecation errors.
  #![allow(deprecated)]

  pub type ClapCommand<'help> = clap::App<'help>;

  pub trait ErrorExt {
    fn kind(&self) -> clap::ErrorKind;
  }

  impl ErrorExt for clap::Error {
    fn kind(&self) -> clap::ErrorKind {
      self.kind
    }
  }
}

use clapfix::{ClapCommand as App, ErrorExt};

/// The resolution of a argument match.
#[derive(Default, Debug, Serialize)]
#[non_exhaustive]
pub struct ArgData {
  /// - [`Value::Bool`] if it's a flag,
  /// - [`Value::Array`] if it's multiple,
  /// - [`Value::String`] if it has value,
  /// - [`Value::Null`] otherwise.
  pub value: Value,
  /// The number of occurrences of the argument.
  /// e.g. `./app --arg 1 --arg 2 --arg 2 3 4` results in three occurrences.
  pub occurrences: u64,
}

/// The matched subcommand.
#[derive(Default, Debug, Serialize)]
#[non_exhaustive]
pub struct SubcommandMatches {
  /// The subcommand name.
  pub name: String,
  /// The subcommand argument matches.
  pub matches: Matches,
}

/// The argument matches of a command.
#[derive(Default, Debug, Serialize)]
#[non_exhaustive]
pub struct Matches {
  /// Data structure mapping each found arg with its resolution.
  pub args: HashMap<String, ArgData>,
  /// The matched subcommand if found.
  pub subcommand: Option<Box<SubcommandMatches>>,
}

impl Matches {
  /// Set a arg match.
  pub(crate) fn set_arg(&mut self, name: String, value: ArgData) {
    self.args.insert(name, value);
  }

  /// Sets the subcommand matches.
  pub(crate) fn set_subcommand(&mut self, name: String, matches: Matches) {
    self.subcommand = Some(Box::new(SubcommandMatches { name, matches }));
  }
}

/// Gets the argument matches of the CLI definition.
///
/// This is a low level API. If the application has been built,
/// prefer [`App::get_cli_matches`](`crate::App#method.get_cli_matches`).
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::api::cli::get_matches;
/// tauri::Builder::default()
///   .setup(|app| {
///     let matches = get_matches(app.config().tauri.cli.as_ref().unwrap(), app.package_info())?;
///     Ok(())
///   });
/// ```
pub fn get_matches(cli: &CliConfig, package_info: &PackageInfo) -> crate::api::Result<Matches> {
  let about = cli
    .description()
    .unwrap_or(&package_info.description.to_string())
    .to_string();
  let version = &*package_info.version.to_string();
  let app = get_app(package_info, version, &package_info.name, Some(&about), cli);
  match app.try_get_matches() {
    Ok(matches) => Ok(get_matches_internal(cli, &matches)),
    Err(e) => match ErrorExt::kind(&e) {
      ErrorKind::DisplayHelp => {
        let mut matches = Matches::default();
        let help_text = e.to_string();
        matches.args.insert(
          "help".to_string(),
          ArgData {
            value: Value::String(help_text),
            occurrences: 0,
          },
        );
        Ok(matches)
      }
      ErrorKind::DisplayVersion => {
        let mut matches = Matches::default();
        matches
          .args
          .insert("version".to_string(), Default::default());
        Ok(matches)
      }
      _ => Err(e.into()),
    },
  }
}

fn get_matches_internal(config: &CliConfig, matches: &ArgMatches) -> Matches {
  let mut cli_matches = Matches::default();
  map_matches(config, matches, &mut cli_matches);

  if let Some((subcommand_name, subcommand_matches)) = matches.subcommand() {
    let mut subcommand_cli_matches = Matches::default();
    map_matches(
      config.subcommands().unwrap().get(subcommand_name).unwrap(),
      subcommand_matches,
      &mut subcommand_cli_matches,
    );
    cli_matches.set_subcommand(subcommand_name.to_string(), subcommand_cli_matches);
  }

  cli_matches
}

fn map_matches(config: &CliConfig, matches: &ArgMatches, cli_matches: &mut Matches) {
  if let Some(args) = config.args() {
    for arg in args {
      let occurrences = matches.occurrences_of(arg.name.clone());
      let value = if occurrences == 0 || !arg.takes_value {
        Value::Bool(occurrences > 0)
      } else if arg.multiple {
        matches
          .values_of(arg.name.clone())
          .map(|v| {
            let mut values = Vec::new();
            for value in v {
              values.push(Value::String(value.to_string()));
            }
            Value::Array(values)
          })
          .unwrap_or(Value::Null)
      } else {
        matches
          .value_of(arg.name.clone())
          .map(|v| Value::String(v.to_string()))
          .unwrap_or(Value::Null)
      };

      cli_matches.set_arg(arg.name.clone(), ArgData { value, occurrences });
    }
  }
}

fn get_app<'a>(
  package_info: &'a PackageInfo,
  version: &'a str,
  command_name: &'a str,
  about: Option<&'a String>,
  config: &'a CliConfig,
) -> App<'a> {
  let mut app = App::new(command_name)
    .author(package_info.authors)
    .version(version);

  if let Some(about) = about {
    app = app.about(&**about);
  }
  if let Some(long_description) = config.long_description() {
    app = app.long_about(&**long_description);
  }
  if let Some(before_help) = config.before_help() {
    app = app.before_help(&**before_help);
  }
  if let Some(after_help) = config.after_help() {
    app = app.after_help(&**after_help);
  }

  if let Some(args) = config.args() {
    for arg in args {
      let arg_name = arg.name.as_ref();
      app = app.arg(get_arg(arg_name, arg));
    }
  }

  if let Some(subcommands) = config.subcommands() {
    for (subcommand_name, subcommand) in subcommands {
      let clap_subcommand = get_app(
        package_info,
        version,
        subcommand_name,
        subcommand.description(),
        subcommand,
      );
      app = app.subcommand(clap_subcommand);
    }
  }

  app
}

fn get_arg<'a>(arg_name: &'a str, arg: &'a CliArg) -> Arg<'a> {
  let mut clap_arg = Arg::new(arg_name);

  if arg.index.is_none() {
    clap_arg = clap_arg.long(arg_name);
    if let Some(short) = arg.short {
      clap_arg = clap_arg.short(short);
    }
  }

  clap_arg = bind_string_arg!(arg, clap_arg, description, help);
  clap_arg = bind_string_arg!(arg, clap_arg, long_description, long_help);
  clap_arg = clap_arg.takes_value(arg.takes_value);
  clap_arg = clap_arg.multiple_values(arg.multiple);
  clap_arg = clap_arg.multiple_occurrences(arg.multiple_occurrences);
  clap_arg = bind_value_arg!(arg, clap_arg, number_of_values);
  clap_arg = bind_string_slice_arg!(arg, clap_arg, possible_values);
  clap_arg = bind_value_arg!(arg, clap_arg, min_values);
  clap_arg = bind_value_arg!(arg, clap_arg, max_values);
  clap_arg = clap_arg.required(arg.required);
  clap_arg = bind_string_arg!(
    arg,
    clap_arg,
    required_unless_present,
    required_unless_present
  );
  clap_arg = bind_string_slice_arg!(arg, clap_arg, required_unless_present_all);
  clap_arg = bind_string_slice_arg!(arg, clap_arg, required_unless_present_any);
  clap_arg = bind_string_arg!(arg, clap_arg, conflicts_with, conflicts_with);
  if let Some(value) = &arg.conflicts_with_all {
    let v: Vec<&str> = value.iter().map(|x| &**x).collect();
    clap_arg = clap_arg.conflicts_with_all(&v);
  }
  clap_arg = bind_string_arg!(arg, clap_arg, requires, requires);
  if let Some(value) = &arg.requires_all {
    let v: Vec<&str> = value.iter().map(|x| &**x).collect();
    clap_arg = clap_arg.requires_all(&v);
  }
  clap_arg = bind_if_arg!(arg, clap_arg, requires_if);
  clap_arg = bind_if_arg!(arg, clap_arg, required_if_eq);
  clap_arg = bind_value_arg!(arg, clap_arg, require_equals);
  clap_arg = bind_value_arg!(arg, clap_arg, index);

  clap_arg
}
