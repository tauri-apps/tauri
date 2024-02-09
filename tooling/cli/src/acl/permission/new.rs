use std::{collections::HashSet, path::PathBuf};

use clap::Parser;

use crate::{
  helpers::{app_paths::tauri_dir_opt, prompts},
  Result,
};

#[derive(serde::Serialize)]
struct Commands {
  allow: HashSet<String>,
  deny: HashSet<String>,
}

#[derive(serde::Serialize)]
struct Permission {
  identifier: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  commands: Commands,
}

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
  /// Use toml for the permission file.
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

  let description = match options.description {
    Some(d) => Some(d),
    None => prompts::input::<String>("What's the permission description?", None, false, true)?
      .and_then(|d| if d.is_empty() { None } else { Some(d) }),
  };

  let allow: HashSet<String> = options
    .allow
    .map(FromIterator::from_iter)
    .unwrap_or_default();
  let deny: HashSet<String> = options
    .deny
    .map(FromIterator::from_iter)
    .unwrap_or_default();

  let permission = Permission {
    identifier,
    description,
    commands: Commands { allow, deny },
  };

  let path = match options.out {
    Some(o) => o.canonicalize()?,
    None => {
      let dir = match tauri_dir_opt() {
        Some(t) => t,
        None => std::env::current_dir()?,
      };
      let permissions_dir = dir.join("permissions");
      let extension = if options.toml { "toml" } else { "conf.json" };
      permissions_dir.join(format!("{}.{extension}", permission.identifier))
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

  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }

  let contents = if options.toml {
    toml_edit::ser::to_string_pretty(&permission)?
  } else {
    serde_json::to_string_pretty(&permission)?
  };
  std::fs::write(&path, contents)?;

  log::info!(action = "Created"; "permission at {}", dunce::simplified(&path).display());

  Ok(())
}
