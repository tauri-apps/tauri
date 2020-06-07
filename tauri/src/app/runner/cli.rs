use crate::config::{Cli, Config};

use clap::{App, Arg, ArgMatches};

#[macro_use]
mod macros;

pub(crate) fn get_matches(config: Config) -> ArgMatches {
  let cli = config.tauri.cli.unwrap();

  let about = cli
    .description()
    .unwrap_or(&crate_description!().to_string())
    .to_string();
  let app = get_app(crate_name!(), Some(&about), &cli);

  app.get_matches()
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
      let mut clap_arg = Arg::new(arg_name)
        .long(arg_name);

      if let Some(short) = arg.short {
        clap_arg = clap_arg.short(short);
      }

      clap_arg = bind_string_arg!(arg, clap_arg, description, about);
      clap_arg = bind_string_arg!(arg, clap_arg, long_description, long_about);
      clap_arg = bind_value_arg!(arg, clap_arg, takes_value);
      clap_arg = bind_value_arg!(arg, clap_arg, multiple);
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
      clap_arg = bind_value_arg!(arg, clap_arg, global);

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
