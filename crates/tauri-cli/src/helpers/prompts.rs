// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Display, str::FromStr};

use crate::Result;

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
    Ok(initial)
  } else {
    let theme = dialoguer::theme::ColorfulTheme::default();
    let mut builder = dialoguer::Input::with_theme(&theme)
      .with_prompt(prompt)
      .allow_empty(allow_empty);

    if let Some(v) = initial {
      builder = builder.with_initial_text(v.to_string());
    }

    builder
      .interact_text()
      .map(|t: T| if t.ne("") { Some(t) } else { None })
      .map_err(Into::into)
  }
}

pub fn confirm(prompt: &str, default: Option<bool>) -> Result<bool> {
  let theme = dialoguer::theme::ColorfulTheme::default();
  let mut builder = dialoguer::Confirm::with_theme(&theme).with_prompt(prompt);
  if let Some(default) = default {
    builder = builder.default(default);
  }
  builder.interact().map_err(Into::into)
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
  builder.interact().map_err(Into::into)
}
