// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Display, str::FromStr};

use crate::{error::Context, Result};

fn strip_surrounding_quotes(input: &str) -> &str {
  let bytes = input.as_bytes();
  if bytes.len() >= 2 {
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
      return &input[1..input.len() - 1];
    }
  }
  input
}

fn validate_common(input: &str) -> std::result::Result<(), String> {
  if let Some(c) = input.chars().find(|c| c.is_control() && *c != '\t') {
    return Err(format!(
      "invalid input: control characters are not allowed (found {c:?})"
    ));
  }
  Ok(())
}

pub fn input<T>(
  prompt: &str,
  initial: Option<T>,
  skip: bool,
  allow_empty: bool,
) -> Result<Option<T>>
where
    T: Clone + FromStr + Display + ToString,
    T::Err: Display + std::fmt::Debug,
    T: PartialEq<str>,
{
  if skip {
    return Ok(initial);
  }

  let theme = dialoguer::theme::ColorfulTheme::default();
  let mut builder = dialoguer::Input::with_theme(&theme)
      .with_prompt(prompt)
      .allow_empty(allow_empty)
      .validate_with(|input: &T| -> std::result::Result<(), String> {
        let raw = input.to_string();
        let normalized = strip_surrounding_quotes(&raw);
        validate_common(normalized)
      });

  if let Some(v) = &initial {
    builder = builder.with_initial_text(v.to_string());
  }

  let value = builder.interact_text().context("failed to prompt input")?;

  let raw = value.to_string();
  let normalized = strip_surrounding_quotes(&raw);

  if normalized.is_empty() {
    return Ok(None);
  }

  if normalized == raw {
    return Ok(Some(value));
  }

  T::from_str(normalized)
      .ok()
      .context(format!(
        "invalid value {normalized:?} (after removing surrounding quotes)"
      ))
      .map(Some)
}

pub fn confirm(prompt: &str, default: Option<bool>) -> Result<bool> {
  let theme = dialoguer::theme::ColorfulTheme::default();
  let mut builder = dialoguer::Confirm::with_theme(&theme).with_prompt(prompt);
  if let Some(default) = default {
    builder = builder.default(default);
  }
  builder.interact().context("failed to prompt confirm")
}

pub fn multiselect<T: ToString>(
  prompt: &str,
  items: &[T],
  defaults: Option<&[bool]>,
) -> Result<Vec<usize>> {
  let theme = dialoguer::theme::ColorfulTheme::default();
  let mut builder = dialoguer::MultiSelect::with_theme(&theme)
      .with_prompt(prompt)
      .items(items);
  if let Some(defaults) = defaults {
    builder = builder.defaults(defaults);
  }
  builder.interact().context("failed to prompt multi-select")
}

pub fn validate_url(value: &str) -> std::result::Result<(), String> {
  if value.trim().is_empty() {
    return Ok(());
  }
  if value.chars().any(char::is_whitespace) {
    return Err(format!("the URL {value:?} must not contain whitespace"));
  }
  if !value.contains("://") {
    return Err(format!(
      "the URL {value:?} is missing a scheme, e.g. http:// or https://"
    ));
  }
  Ok(())
}