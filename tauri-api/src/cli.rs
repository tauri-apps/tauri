use crate::config::{get as get_config, Cli};

use clap::{App, Arg, ArgMatches};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[macro_use]
mod macros;

#[derive(Default, Debug, Serialize)]
pub struct ArgData {
  value: Value,
  occurrences: u64,
}

#[derive(Default, Debug, Serialize)]
pub struct SubcommandMatches {
  name: String,
  matches: Matches,
}

#[derive(Default, Debug, Serialize)]
pub struct Matches {
  args: HashMap<String, ArgData>,
  subcommand: Option<Box<SubcommandMatches>>,
}

impl Matches {
  pub(crate) fn set_arg(&mut self, name: String, value: ArgData) {
    self.args.insert(name, value);
  }

  pub(crate) fn set_subcommand(&mut self, name: String, matches: Matches) {
    self.subcommand = Some(Box::new(SubcommandMatches { name, matches }));
  }
}

pub fn get_matches() -> crate::Result<Matches> {
  let config = get_config()?;
  let cli = config
    .tauri
    .cli
    .as_ref()
    .ok_or(anyhow::anyhow!("CLI configuration not defined"))?;

  let about = cli
    .description()
    .unwrap_or(&crate_description!().to_string())
    .to_string();
  let app = get_app(crate_name!(), Some(&about), cli);
  let matches = app.get_matches();
  Ok(get_matches_internal(cli, &matches))
}

fn get_matches_internal<T: Cli + 'static>(config: &T, matches: &ArgMatches) -> Matches {
  let mut cli_matches = Matches::default();
  map_matches(config, matches, &mut cli_matches);

  let (subcommand_name, subcommand_matches_option) = matches.subcommand();
  if let Some(subcommand_matches) = subcommand_matches_option {
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

fn map_matches<T: Cli + 'static>(config: &T, matches: &ArgMatches, cli_matches: &mut Matches) {
  if let Some(args) = config.args() {
    for arg in args {
      let occurrences = matches.occurrences_of(arg.name.clone());
      let value = if occurrences == 0 || !arg.takes_value.unwrap_or(false) {
        Value::Null
      } else if arg.multiple.unwrap_or(false) {
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

fn get_app<'a, T: Cli + 'static>(name: &str, about: Option<&'a String>, config: &'a T) -> App<'a> {
  let mut app = App::new(name)
    .author(crate_authors!())
    .version(crate_version!());

  if let Some(about) = about {
    app = app.about(&**about);
  }
  if let Some(long_description) = config.long_description() {
    app = app.long_about(&**long_description);
  }

  if let Some(args) = config.args() {
    for arg in args {
      let arg_name = arg.name.as_ref();
      let mut clap_arg = Arg::new(arg_name).long(arg_name);

      if let Some(short) = arg.short {
        clap_arg = clap_arg.short(short);
      }

      clap_arg = bind_string_arg!(arg, clap_arg, description, about);
      clap_arg = bind_string_arg!(arg, clap_arg, long_description, long_about);
      clap_arg = bind_value_arg!(arg, clap_arg, takes_value);
      clap_arg = bind_value_arg!(arg, clap_arg, multiple);
      clap_arg = bind_value_arg!(arg, clap_arg, multiple_occurrences);
      clap_arg = bind_value_arg!(arg, clap_arg, number_of_values);
      clap_arg = bind_string_slice_arg!(arg, clap_arg, possible_values);
      clap_arg = bind_value_arg!(arg, clap_arg, min_values);
      clap_arg = bind_value_arg!(arg, clap_arg, max_values);
      clap_arg = bind_string_arg!(arg, clap_arg, required_unless, required_unless);
      clap_arg = bind_value_arg!(arg, clap_arg, required);
      clap_arg = bind_string_arg!(arg, clap_arg, required_unless, required_unless);
      clap_arg = bind_string_slice_arg!(arg, clap_arg, required_unless_all);
      clap_arg = bind_string_slice_arg!(arg, clap_arg, required_unless_one);
      clap_arg = bind_string_arg!(arg, clap_arg, conflicts_with, conflicts_with);
      clap_arg = bind_string_slice_arg!(arg, clap_arg, conflicts_with_all);
      clap_arg = bind_string_arg!(arg, clap_arg, requires, requires);
      clap_arg = bind_string_slice_arg!(arg, clap_arg, requires_all);
      clap_arg = bind_if_arg!(arg, clap_arg, requires_if);
      clap_arg = bind_if_arg!(arg, clap_arg, required_if);
      clap_arg = bind_value_arg!(arg, clap_arg, require_equals);
      clap_arg = bind_value_arg!(arg, clap_arg, index);

      app = app.arg(clap_arg);
    }
  }

  if let Some(subcommands) = config.subcommands() {
    for (subcommand_name, subcommand) in subcommands {
      let clap_subcommand = get_app(&subcommand_name, subcommand.description(), subcommand);
      app = app.subcommand(clap_subcommand);
    }
  }

  app
}
