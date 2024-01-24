use std::collections::HashSet;

use clap::{ArgAction, Parser};

use crate::{
  helpers::{app_paths::tauri_dir, prompts},
  Result,
};

#[derive(Debug, Parser)]
#[clap(about = "Create a new custom permission")]
pub struct Options {
  /// Permission identifier.
  identifier: Option<String>,
  /// Permission description
  #[clap(long)]
  description: Option<String>,
  /// List of commands to allow
  #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
  allow: Option<Vec<String>>,
  /// List of commands to deny
  #[clap(short, long, use_value_delimiter = true, value_delimiter = ',')]
  deny: Option<Vec<String>>,
  /// Add a scope config.
  #[clap(long)]
  scope: Option<bool>,
  /// Create inlined in `tauri.conf.json` or `Tauri.toml`.
  #[clap(long)]
  inline: Option<bool>,
  /// If not inlined, use toml for the dedicated permission file.
  #[clap(long)]
  toml: bool,
}

pub fn command(options: Options) -> Result<()> {
  dbg!(&options);

  let identifier = match options.identifier {
    Some(i) => i,
    None => prompts::input("What's the permission identifier?", None, false, false)?.unwrap(),
  };
  let description = options.description;
  let allow: Option<HashSet<String>> = options.allow.map(FromIterator::from_iter);
  let deny: Option<HashSet<String>> = options.deny.map(FromIterator::from_iter);
  let scope = match options.scope {
    Some(i) => i,
    None => prompts::confirm("Create a scope config?", Some(false))?,
  };
  let inline = match options.inline {
    Some(i) => i,
    None => prompts::confirm("Should it be inlined in tauri.conf.json?", Some(false))?,
  };

  let tauri_dir = tauri_dir();

  if !inline {
    let acl = tauri_dir.join("acl");
    std::fs::create_dir_all(&acl)?;

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

    let (extension, contents) = if options.toml {
      let description = match description {
        Some(d) => format!("\ndescription = \"{d}\""),
        None => String::new(),
      };

      let commands = format!("\n\n[commands]\nallow = [{allow}]\ndeny  = [{deny}]");

      let scope = if scope { "\n\n[scope]\nallow = []" } else { "" };

      let contents = format!("identifier = \"{identifier}\"{description}{commands}{scope}");
      ("toml", contents)
    } else {
      let description = match description {
        Some(d) => format!(",\n  \"description\": \"{d}\""),
        None => String::new(),
      };

      let commands =
        format!(",\n  \"commands\": {{\n    \"allow\": [{allow}],\n    \"deny\": [{deny}]\n  }}");

      let scope = if scope {
        ",\n  \"scope\": {\n    \"allow\": []\n  }"
      } else {
        ""
      };

      let contents =
        format!("{{\n  \"identifier\": \"{identifier}\"{description}{commands}{scope}\n}}");
      ("conf.json", contents)
    };
    let path = acl.join(format!("permission.{identifier}.{extension}"));

    if path.exists() {
      anyhow::bail!("Permission already exists at {}", path.display());
    }

    std::fs::write(path, contents)?;
  } else {
    todo!()
  }

  Ok(())
}
