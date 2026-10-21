// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Output of the values that must be configured in CI/CD platforms to sign apps.

use std::{
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
};

use clap::Args;

use crate::{
  Result,
  error::{Context, ErrorExt},
};

/// A value the CLI reads from an environment variable when building in CI/CD platforms.
pub struct Secret {
  pub name: &'static str,
  pub value: String,
}

impl Secret {
  pub fn new(name: &'static str, value: impl Into<String>) -> Self {
    Self {
      name,
      value: value.into(),
    }
  }
}

#[derive(Debug, Clone, Args)]
pub struct SecretsOptions {
  /// Store the CI/CD values as GitHub Actions secrets using the GitHub CLI (`gh secret set`)
  #[clap(long)]
  github_secrets: bool,
  /// Repository to store the GitHub Actions secrets in, in the `OWNER/REPO` format.
  /// Defaults to the repository of the current directory.
  #[clap(long, requires = "github_secrets")]
  github_repo: Option<String>,
  /// Append the CI/CD values to the given dotenv file
  #[clap(long)]
  env_file: Option<PathBuf>,
}

impl SecretsOptions {
  /// Prints, stores or writes the secrets, depending on the options.
  ///
  /// Values are only printed when they are not stored anywhere else.
  pub fn output(&self, secrets: &[Secret]) -> Result<()> {
    if self.github_secrets {
      set_github_secrets(secrets, self.github_repo.as_deref())?;
    }

    if let Some(env_file) = &self.env_file {
      write_env_file(env_file, secrets)?;
      log::info!(
        "Wrote {} to {}",
        names(secrets),
        tauri_utils::display_path(env_file)
      );
    }

    if !self.github_secrets && self.env_file.is_none() {
      println!();
      println!(
        "Set the following environment variables in your CI/CD platform (keep them secret!):"
      );
      for secret in secrets {
        println!();
        println!("{}:", secret.name);
        println!("{}", secret.value);
      }
      println!();
      println!(
        "Tip: pass --github-secrets to store them as GitHub Actions secrets or --env-file <PATH> to write them to a dotenv file."
      );
    }

    Ok(())
  }
}

fn names(secrets: &[Secret]) -> String {
  secrets
    .iter()
    .map(|s| s.name)
    .collect::<Vec<_>>()
    .join(", ")
}

fn set_github_secrets(secrets: &[Secret], repo: Option<&str>) -> Result<()> {
  let gh = which::which("gh").context(
    "the GitHub CLI (`gh`) is required to store GitHub Actions secrets; see https://cli.github.com",
  )?;

  for secret in secrets {
    let mut cmd = Command::new(&gh);
    cmd.args(["secret", "set", secret.name]);
    if let Some(repo) = repo {
      cmd.args(["--repo", repo]);
    }
    // the value is read from stdin so it does not show up in the process list
    let mut child =
      cmd
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| crate::Error::CommandFailed {
          command: "gh secret set".into(),
          error,
        })?;
    child
      .stdin
      .take()
      .expect("stdin is piped")
      .write_all(secret.value.as_bytes())
      .context("failed to write secret to gh")?;
    let status = child.wait().map_err(|error| crate::Error::CommandFailed {
      command: "gh secret set".into(),
      error,
    })?;
    if !status.success() {
      crate::error::bail!("failed to set GitHub secret {}", secret.name);
    }
  }

  Ok(())
}

fn write_env_file(path: &std::path::Path, secrets: &[Secret]) -> Result<()> {
  let mut options = std::fs::OpenOptions::new();
  options.create(true).append(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  let mut file = options
    .open(path)
    .fs_context("failed to open env file", path.to_path_buf())?;

  let mut contents = String::new();
  for secret in secrets {
    contents.push_str(&env_line(secret));
    contents.push('\n');
  }
  file
    .write_all(contents.as_bytes())
    .fs_context("failed to write env file", path.to_path_buf())
}

fn env_line(secret: &Secret) -> String {
  let escaped = secret
    .value
    .replace('\\', "\\\\")
    .replace('"', "\\\"")
    .replace('\n', "\\n");
  format!("{}=\"{escaped}\"", secret.name)
}

/// Generates a random alphanumeric password.
pub fn random_password() -> String {
  use rand::distr::{Alphanumeric, SampleString};
  Alphanumeric.sample_string(&mut rand::rng(), 32)
}

pub fn base64(data: &[u8]) -> String {
  use base64::Engine;
  base64::engine::general_purpose::STANDARD.encode(data)
}

/// Reads a password from the given value, prompts for it or generates a random one in CI.
pub fn password(value: Option<String>, prompt: &str, confirm: bool, ci: bool) -> Result<String> {
  if let Some(value) = value {
    return Ok(value);
  }
  if ci {
    let password = random_password();
    log::info!("Generated a random password");
    return Ok(password);
  }
  let theme = dialoguer::theme::ColorfulTheme::default();
  let mut builder = dialoguer::Password::with_theme(&theme).with_prompt(prompt);
  if confirm {
    builder = builder.with_confirmation("Confirm password", "Passwords do not match");
  }
  builder.interact().context("failed to prompt password")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn escapes_env_values() {
    assert_eq!(
      env_line(&Secret::new("PASSWORD", r#"a"b\c"#)),
      r#"PASSWORD="a\"b\\c""#
    );
  }

  #[test]
  fn appends_env_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    write_env_file(&path, &[Secret::new("A", "1")]).unwrap();
    write_env_file(&path, &[Secret::new("B", "2")]).unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), "A=\"1\"\nB=\"2\"\n");
  }
}
