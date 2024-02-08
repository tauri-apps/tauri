use std::{collections::HashSet, path::PathBuf};

use clap::Parser;

use crate::{
  helpers::{app_paths::tauri_dir_opt, prompts},
  Result,
};

#[derive(Debug, Parser)]
#[clap(about = "Create a new permission file")]
pub struct Options {
  /// Permission identifier.
  identifier: Option<String>,
  /// Permission description
  #[clap(long)]
  description: Option<String>,
  /// List of commands to allow
  #[clap(short, long, use_value_delimiter = true)]
  allow: Option<Vec<String>>,
  /// List of commands to deny
  #[clap(short, long, use_value_delimiter = true)]
  deny: Option<Vec<String>>,
  /// Use toml for the dedicated permission file.
  #[clap(long)]
  toml: bool,
  /// The output file.
  #[clap(short, long)]
  out: Option<PathBuf>,
}

pub fn command(options: Options) -> Result<()> {
  let identifier = match options.identifier {
    Some(i) => i,
    None => prompts::input("What's the permission identifier?", None, false, false)?.unwrap(),
  };
  let allow: Option<HashSet<String>> = options.allow.map(FromIterator::from_iter);
  let deny: Option<HashSet<String>> = options.deny.map(FromIterator::from_iter);

  let path = match options.out {
    Some(o) => o.canonicalize()?,
    None => {
      let dir = match tauri_dir_opt() {
        Some(t) => t,
        None => std::env::current_dir()?,
      };
      let permissions_dir = dir.join("permissions");
      let extension = if options.toml { "toml" } else { "conf.json" };
      permissions_dir.join(format!("{identifier}.{extension}"))
    }
  };

  if path.exists() {
    let msg = format!(
      "Permission already exists at {}",
      dunce::simplified(&path).display()
    );
    let overwrite = prompts::confirm(&format!("{msg}, overwrite?"), Some(false))?;
    if overwrite {
      std::fs::remove_file(&path)?;
    } else {
      anyhow::bail!(msg);
    }
  }

  let allow = allow
    .unwrap_or_default()
    .into_iter()
    .map(|c| format!("\"{c}\""))
    .collect::<Vec<_>>()
    .join(", ");

  let deny = deny
    .unwrap_or_default()
    .into_iter()
    .map(|c| format!("\"{c}\""))
    .collect::<Vec<_>>()
    .join(", ");

  let contents = if options.toml {
    let description = match options.description {
      Some(d) => format!("\ndescription = \"{d}\""),
      None => String::new(),
    };
    format!("identifier = \"{identifier}\"{description}\n\n[commands]\nallow = [{allow}]\ndeny = [{deny}]")
  } else {
    let description = match options.description {
      Some(d) => format!(",\n  \"description\": \"{d}\""),
      None => String::new(),
    };
    format!("{{\n  \"identifier\": \"{identifier}\"{description},\n  \"commands\": {{\n    \"allow\": [{allow}],\n    \"deny\": [{deny}]\n  }}\n}}")
  };

  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }
  std::fs::write(path, contents)?;

  Ok(())
}
