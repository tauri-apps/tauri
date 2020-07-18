pub use anyhow::Result;
use clap::{load_yaml, App, AppSettings, ArgMatches};
use std::convert::TryInto;

mod build;
mod dev;
mod helpers;
mod info;
mod init;

pub use helpers::Logger;

fn init_command(matches: &ArgMatches) -> Result<()> {
  let force = matches.value_of("force");
  let directory = matches.value_of("directory");
  let tauri_path = matches.value_of("tauri_path");
  let app_name = matches.value_of("app_name");
  let window_title = matches.value_of("window_title");
  let dist_dir = matches.value_of("dist_dir");
  let dev_path = matches.value_of("dev_path");

  let mut init_runner = init::Init::new();
  if let Some(force) = force {
    init_runner = init_runner.force(force.try_into()?);
  }
  if let Some(directory) = directory {
    init_runner = init_runner.directory(directory);
  }
  if let Some(tauri_path) = tauri_path {
    init_runner = init_runner.tauri_path(tauri_path);
  }
  if let Some(app_name) = app_name {
    init_runner = init_runner.app_name(app_name);
  }
  if let Some(window_title) = window_title {
    init_runner = init_runner.window_title(window_title);
  }
  if let Some(dist_dir) = dist_dir {
    init_runner = init_runner.dist_dir(dist_dir);
  }
  if let Some(dev_path) = dev_path {
    init_runner = init_runner.directory(dev_path);
  }

  init_runner.run()
}

fn dev_command(matches: &ArgMatches) -> Result<()> {
  let exit_on_panic = matches.is_present("exit-on-panic");
  dev::Dev::new().exit_on_panic(exit_on_panic).run()
}

fn build_command(matches: &ArgMatches) -> Result<()> {
  let debug = matches.is_present("debug");
  let targets = matches.values_of_lossy("target");

  let mut build_runner = build::Build::new();
  if debug {
    build_runner = build_runner.debug();
  }
  if let Some(targets) = targets {
    build_runner = build_runner.targets(targets);
  }

  build_runner.run()
}

fn info_command() -> Result<()> {
  info::Info::new().run()
}

fn main() -> Result<()> {
  let yaml = load_yaml!("cli.yml");
  let app = App::from(yaml).setting(AppSettings::ArgRequiredElseHelp);
  let matches = app.get_matches();

  if let Some(matches) = matches.subcommand_matches("init") {
    init_command(&matches)?;
  } else if let Some(matches) = matches.subcommand_matches("dev") {
    dev_command(&matches)?;
  } else if let Some(matches) = matches.subcommand_matches("build") {
    build_command(&matches)?;
  } else if matches.subcommand_matches("info").is_some() {
    info_command()?;
  }

  Ok(())
}
